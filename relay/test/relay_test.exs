defmodule RelayTest do
  use ExUnit.Case, async: false
  doctest Relay

  # Note: We don't need to alias SessionStore since we use Relay module functions

  setup do
    # SessionStore is already started by the application
    # Just ensure it's running
    Process.whereis(Relay.SessionStore) || raise "SessionStore not started"
    :ok
  end

  describe "create_session/2" do
    test "creates session with valid data and TTL" do
      data = %{"proof" => "age_over_21", "timestamp" => DateTime.utc_now()}

      assert {:ok, session_id} = Relay.create_session(data, 60)
      assert is_binary(session_id)
      assert String.length(session_id) == 36  # UUID v4 length
    end

    test "creates multiple unique sessions" do
      assert {:ok, id1} = Relay.create_session(%{"data" => "test1"}, 60)
      assert {:ok, id2} = Relay.create_session(%{"data" => "test2"}, 60)
      assert id1 != id2
    end

    test "rejects invalid TTL values" do
      assert {:error, :invalid_ttl} = Relay.create_session(%{}, 0)
      assert {:error, :invalid_ttl} = Relay.create_session(%{}, -1)
      assert {:error, :invalid_ttl} = Relay.create_session(%{}, "invalid")
    end

    test "handles large data payloads" do
      large_data = %{"payload" => String.duplicate("x", 10_000)}
      assert {:ok, _session_id} = Relay.create_session(large_data, 60)
    end
  end

  describe "get_session_data/1" do
    test "retrieves and destroys session data" do
      data = %{"proof" => "identity_verified"}
      {:ok, session_id} = Relay.create_session(data, 60)

      assert {:ok, ^data} = Relay.get_session_data(session_id)

      # Session should be destroyed after retrieval
      assert {:error, :not_found} = Relay.get_session_data(session_id)
    end

    test "returns error for nonexistent session" do
      fake_id = UUID.uuid4()
      assert {:error, :not_found} = Relay.get_session_data(fake_id)
    end

    test "returns error for expired session" do
      data = %{"test" => "expires_quickly"}
      {:ok, session_id} = Relay.create_session(data, 1)  # 1 second TTL

      # Wait for expiration
      Process.sleep(1100)

      assert {:error, :expired} = Relay.get_session_data(session_id)
    end

    test "rejects invalid session ID formats" do
      assert {:error, :invalid_session_id} = Relay.get_session_data(nil)
      assert {:error, :invalid_session_id} = Relay.get_session_data(123)
      assert {:error, :invalid_session_id} = Relay.get_session_data("invalid")
    end
  end

  describe "session_exists?/1" do
    test "returns true for existing session" do
      {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 60)
      assert Relay.session_exists?(session_id) == true
    end

    test "returns false for nonexistent session" do
      fake_id = UUID.uuid4()
      assert Relay.session_exists?(fake_id) == false
    end

    test "returns false for expired session" do
      {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 1)
      Process.sleep(1100)  # Wait for expiration
      assert Relay.session_exists?(session_id) == false
    end

    test "returns false for invalid session ID formats" do
      assert Relay.session_exists?(nil) == false
      assert Relay.session_exists?(123) == false
      assert Relay.session_exists?("") == false
    end
  end

  describe "destroy_session/1" do
    test "manually destroys existing session" do
      {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 60)

      assert :ok = Relay.destroy_session(session_id)
      assert Relay.session_exists?(session_id) == false
    end

    test "returns error for nonexistent session" do
      fake_id = UUID.uuid4()
      assert {:error, :not_found} = Relay.destroy_session(fake_id)
    end

    test "rejects invalid session ID formats" do
      assert {:error, :invalid_session_id} = Relay.destroy_session(nil)
      assert {:error, :invalid_session_id} = Relay.destroy_session(123)
    end
  end

  describe "stats/0" do
    test "returns session statistics" do
      # Create some test sessions
      {:ok, _id1} = Relay.create_session(%{"test" => "data1"}, 60)
      {:ok, _id2} = Relay.create_session(%{"test" => "data2"}, 60)
      {:ok, _id3} = Relay.create_session(%{"test" => "data3"}, 1)  # Will expire quickly

      Process.sleep(1100)  # Wait for one session to expire

      stats = Relay.stats()

      assert is_map(stats)
      assert Map.has_key?(stats, :total_sessions)
      assert Map.has_key?(stats, :active_sessions)
      assert Map.has_key?(stats, :expired_sessions)
      assert Map.has_key?(stats, :memory_bytes)

      assert stats.total_sessions >= 2
      assert stats.active_sessions >= 2
      assert stats.memory_bytes > 0
    end
  end

  describe "concurrency and performance" do
    test "handles concurrent session creation" do
      # Create multiple sessions concurrently
      tasks = for i <- 1..100 do
        Task.async(fn ->
          Relay.create_session(%{"concurrent_test" => i}, 60)
        end)
      end

      results = Enum.map(tasks, &Task.await/1)

      # All should succeed and be unique
      assert length(results) == 100
      session_ids = Enum.map(results, fn {:ok, id} -> id end)
      assert length(Enum.uniq(session_ids)) == 100
    end

    test "handles concurrent read/write operations" do
      # Create a session
      {:ok, session_id} = Relay.create_session(%{"concurrent" => "access"}, 60)

      # Try to access it concurrently (only one should succeed)
      tasks = for _i <- 1..10 do
        Task.async(fn ->
          Relay.get_session_data(session_id)
        end)
      end

      results = Enum.map(tasks, &Task.await/1)

      # Exactly one should succeed, others should fail
      success_count = Enum.count(results, fn result ->
        match?({:ok, _}, result)
      end)

      assert success_count == 1
    end

    test "performance benchmark for session operations" do
      # Benchmark session creation
      {creation_time, session_ids} = :timer.tc(fn ->
        for i <- 1..1000 do
          {:ok, session_id} = Relay.create_session(%{"benchmark" => i}, 60)
          session_id
        end
      end)

      # Should create 1000 sessions in under 100ms
      assert creation_time < 100_000  # 100ms in microseconds
      assert length(session_ids) == 1000

      # Benchmark session lookup
      sample_ids = Enum.take_random(session_ids, 100)

      {lookup_time, _results} = :timer.tc(fn ->
        Enum.map(sample_ids, &Relay.session_exists?/1)
      end)

      # Should check 100 sessions in under 10ms
      assert lookup_time < 10_000  # 10ms in microseconds
    end
  end
end
