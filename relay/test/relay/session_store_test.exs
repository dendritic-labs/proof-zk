defmodule Relay.SessionStoreTest do
  use ExUnit.Case, async: false

  alias Relay.SessionStore

  setup do
    # SessionStore is already started by the application
    # Just ensure it's running and ETS table exists
    Process.whereis(Relay.SessionStore) || raise "SessionStore not started"
    :ets.info(:relay_sessions) || raise "ETS table not initialized"
    :ok
  end

  describe "ETS table management" do
    test "initializes ETS table with correct options" do
      # Check that the table exists and has correct properties
      info = :ets.info(:relay_sessions)

      assert info != :undefined
      assert info[:type] == :set
      assert info[:protection] == :public
      assert info[:read_concurrency] == true
      assert info[:write_concurrency] == true
    end

    test "survives process restart" do
      # Create a session
      {:ok, _session_id} = SessionStore.create_session(%{"test" => "data"}, 60)

      # Stop the SessionStore process (if it's running standalone)
      # In a supervised environment, we need to stop the supervisor to fully restart
      pid = Process.whereis(SessionStore)
      if pid do
        GenServer.stop(SessionStore, :normal)
        # Give it a moment to shut down
        Process.sleep(10)
      end

      # Start it again (or let supervisor restart it)
      case start_supervised(SessionStore) do
        {:ok, _pid} -> :ok
        {:error, {:already_started, _pid}} -> :ok  # Already restarted by supervisor
      end

      # Table should be recreated (sessions lost, but table available)
      info = :ets.info(:relay_sessions)
      assert info != :undefined
    end
  end

  describe "session lifecycle" do
    test "creates session with proper expiration time" do
      ttl_seconds = 120
      start_time = System.monotonic_time(:millisecond)

      {:ok, session_id} = SessionStore.create_session(%{"test" => "data"}, ttl_seconds)

      # Check session exists in ETS with correct expiration
      [{^session_id, _data, expires_at}] = :ets.lookup(:relay_sessions, session_id)

      expected_expiry = start_time + (ttl_seconds * 1000)

      # Allow for small timing differences (within 100ms)
      assert abs(expires_at - expected_expiry) < 100
    end

    test "session data integrity" do
      complex_data = %{
        "user_id" => "12345",
        "proof_type" => "age_verification",
        "metadata" => %{
          "timestamp" => DateTime.utc_now(),
          "app_id" => "airline_demo"
        },
        "nested_list" => [1, 2, %{"key" => "value"}]
      }

      {:ok, session_id} = SessionStore.create_session(complex_data, 60)
      {:ok, retrieved_data} = SessionStore.get_and_destroy_session(session_id)

      assert retrieved_data == complex_data
    end
  end

  describe "atomic operations" do
    test "get_and_destroy is atomic" do
      data = %{"atomic_test" => "data"}
      {:ok, session_id} = SessionStore.create_session(data, 60)

      # First access should succeed and destroy
      assert {:ok, ^data} = SessionStore.get_and_destroy_session(session_id)

      # Second access should fail
      assert {:error, :not_found} = SessionStore.get_and_destroy_session(session_id)

      # Verify session no longer exists
      refute SessionStore.session_exists?(session_id)
    end

    test "concurrent access to same session" do
      {:ok, session_id} = SessionStore.create_session(%{"concurrent" => "test"}, 60)

      # Multiple processes try to access the same session
      tasks = for i <- 1..20 do
        Task.async(fn ->
          {i, SessionStore.get_and_destroy_session(session_id)}
        end)
      end

      results = Enum.map(tasks, &Task.await/1)

      # Exactly one should succeed
      successful = Enum.filter(results, fn {_i, result} -> match?({:ok, _}, result) end)
      failed = Enum.filter(results, fn {_i, result} -> match?({:error, _}, result) end)

      assert length(successful) == 1
      assert length(failed) == 19
    end
  end

  describe "expiration handling" do
    test "expired sessions return error on access" do
      {:ok, session_id} = SessionStore.create_session(%{"expires" => "soon"}, 1)

      # Session should exist initially
      assert SessionStore.session_exists?(session_id)

      # Wait for expiration
      Process.sleep(1100)

      # Access should fail
      assert {:error, :expired} = SessionStore.get_and_destroy_session(session_id)
      refute SessionStore.session_exists?(session_id)
    end

    test "cleanup removes expired sessions" do
      # Create sessions with different TTLs
      {:ok, short_id} = SessionStore.create_session(%{"short" => "ttl"}, 1)
      {:ok, long_id} = SessionStore.create_session(%{"long" => "ttl"}, 300)

      # Wait for short session to expire
      Process.sleep(1100)

      # Trigger cleanup (normally done by background process)
      GenServer.cast(SessionStore, :cleanup_expired)
      # Wait (with retries) until cleanup has actually removed the expired session
      wait_for_cleanup =
        fn wait_for_cleanup, attempts_left ->
          case SessionStore.get_and_destroy_session(short_id) do
            {:error, :not_found} ->
              # Short session has been cleaned up; long session should still exist
              assert {:ok, %{"long" => "ttl"}} = SessionStore.get_and_destroy_session(long_id)
            {:error, :expired} when attempts_left > 0 ->
              # Cleanup has not yet run; wait a bit and retry
              Process.sleep(50)
              wait_for_cleanup.(wait_for_cleanup, attempts_left - 1)
            other ->
              flunk("Unexpected result while waiting for cleanup: #{inspect(other)}")
          end
        end
      # Allow up to ~1 second total (20 * 50ms) for cleanup to complete
      wait_for_cleanup.(wait_for_cleanup, 20)
    end
  end

  describe "statistics and monitoring" do
    test "stats reflect current state" do
      # Start with empty state
      initial_stats = SessionStore.stats()

      # Create some sessions
      {:ok, _id1} = SessionStore.create_session(%{"stats1" => "test"}, 60)
      {:ok, _id2} = SessionStore.create_session(%{"stats2" => "test"}, 1)  # Expires quickly
      {:ok, _id3} = SessionStore.create_session(%{"stats3" => "test"}, 60)

      # Wait for one to expire
      Process.sleep(1100)

      stats = SessionStore.stats()

      # Should have more sessions than initially
      assert stats.total_sessions >= initial_stats.total_sessions + 2
      assert stats.expired_sessions >= 1
      assert stats.active_sessions >= 2
      assert stats.memory_bytes > initial_stats.memory_bytes
    end
  end

  describe "error handling and edge cases" do
    test "handles empty data" do
      assert {:ok, session_id} = SessionStore.create_session(%{}, 60)
      assert {:ok, %{}} = SessionStore.get_and_destroy_session(session_id)
    end

    test "handles nil values in data" do
      data = %{"null_value" => nil, "other" => "data"}
      assert {:ok, session_id} = SessionStore.create_session(data, 60)
      assert {:ok, ^data} = SessionStore.get_and_destroy_session(session_id)
    end

    test "destroy nonexistent session" do
      fake_id = UUID.uuid4()
      assert {:error, :not_found} = SessionStore.destroy_session(fake_id)
    end
  end

  describe "performance and scalability" do
    test "handles high session volume" do
      # Create many sessions quickly
      {time_microseconds, session_ids} = :timer.tc(fn ->
        for i <- 1..2000 do
          {:ok, session_id} = SessionStore.create_session(%{"perf_test" => i}, 60)
          session_id
        end
      end)

      # Should complete in reasonable time (less than 1 second)
      assert time_microseconds < 1_000_000
      assert length(session_ids) == 2000

      # Verify stats
      stats = SessionStore.stats()
      assert stats.total_sessions >= 2000
    end

    test "lookup performance remains consistent" do
      # Create baseline sessions
      session_ids = for i <- 1..1000 do
        {:ok, session_id} = SessionStore.create_session(%{"lookup_test" => i}, 60)
        session_id
      end

      # Measure lookup time
      sample_ids = Enum.take_random(session_ids, 100)

      {lookup_time, _results} = :timer.tc(fn ->
        Enum.map(sample_ids, &SessionStore.session_exists?/1)
      end)

      # Should be very fast (under 5ms for 100 lookups)
      assert lookup_time < 5_000
    end
  end
end
