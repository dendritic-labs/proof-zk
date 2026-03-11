defmodule Relay.SessionStore do
  @moduledoc """
  High-performance session storage using ETS for scalability.

  Manages millions of concurrent ephemeral sessions with:
  - Microsecond lookups via ETS
  - Atomic read-and-delete operations
  - Concurrent read/write optimization
  - Memory-efficient storage
  """

  use GenServer
  require Logger

  @table_name :relay_sessions
  @cleanup_interval :timer.minutes(1)

  # Public API

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @doc """
  Create a new ephemeral session.
  Returns {:ok, session_id} or {:error, reason}
  """
  def create_session(data, ttl_seconds) do
    session_id = UUID.uuid4()
    expires_at = System.monotonic_time(:millisecond) + (ttl_seconds * 1000)

    case :ets.insert(@table_name, {session_id, data, expires_at}) do
      true ->
        Logger.debug("Created session #{session_id}, expires in #{ttl_seconds}s")
        {:ok, session_id}
      false ->
        {:error, :insert_failed}
    end
  end

  @doc """
  Retrieve and atomically delete session data.
  Returns {:ok, data} or {:error, reason}
  """
  def get_and_destroy_session(session_id) do
    case :ets.take(@table_name, session_id) do
      [{^session_id, data, expires_at}] ->
        now = System.monotonic_time(:millisecond)
        if now < expires_at do
          Logger.debug("Session #{session_id} data retrieved and destroyed")
          {:ok, data}
        else
          Logger.debug("Session #{session_id} expired during retrieval")
          {:error, :expired}
        end
      [] ->
        {:error, :not_found}
    end
  end

  @doc """
  Check if session exists without consuming it.
  """
  def session_exists?(session_id) do
    case :ets.lookup(@table_name, session_id) do
      [{^session_id, _data, expires_at}] ->
        now = System.monotonic_time(:millisecond)
        now < expires_at
      [] ->
        false
    end
  end

  @doc """
  Manually destroy a session.
  """
  def destroy_session(session_id) do
    case :ets.member(@table_name, session_id) do
      true ->
        :ets.delete(@table_name, session_id)
        Logger.debug("Session #{session_id} manually destroyed")
        :ok
      false ->
        {:error, :not_found}
    end
  end

  @doc """
  Get statistics about current sessions.
  """
  def stats do
    info = :ets.info(@table_name)
    now = System.monotonic_time(:millisecond)

    # Count expired sessions
    expired_count = :ets.select_count(@table_name, [
      {{:_, :_, :"$1"}, [{:<, :"$1", now}], [true]}
    ])

    %{
      total_sessions: info[:size],
      expired_sessions: expired_count,
      active_sessions: info[:size] - expired_count,
      memory_words: info[:memory],
      memory_bytes: info[:memory] * :erlang.system_info(:wordsize)
    }
  end

  # GenServer Callbacks

  @impl true
  def init([]) do
    # Create ETS table with optimized settings for high concurrency
    table = :ets.new(@table_name, [
      :set,                    # Unique keys
      :public,                 # Allow direct access from other processes
      :named_table,           # Use atom name for easy access
      {:read_concurrency, true},   # Optimize for concurrent reads
      {:write_concurrency, true},  # Optimize for concurrent writes
      {:decentralized_counters, true}  # Better performance for info/1
    ])

    Logger.info("SessionStore initialized with ETS table #{@table_name}")

    schedule_cleanup()
    {:ok, %{table: table}}
  end

  @impl true
  def handle_cast(:cleanup_expired, state) do
    cleanup_expired_sessions()
    {:noreply, state}
  end

  @impl true
  def handle_info(:cleanup_expired, state) do
    cleanup_expired_sessions()
    schedule_cleanup()
    {:noreply, state}
  end

  # Private Functions

  defp schedule_cleanup do
    Process.send_after(self(), :cleanup_expired, @cleanup_interval)
  end

  defp cleanup_expired_sessions do
    now = System.monotonic_time(:millisecond)

    # Select and delete expired sessions in batches for efficiency
    expired_sessions = :ets.select(@table_name, [
      {{:"$1", :_, :"$2"}, [{:<, :"$2", now}], [:"$1"]}
    ])

    Enum.each(expired_sessions, fn session_id ->
      :ets.delete(@table_name, session_id)
    end)

    if length(expired_sessions) > 0 do
      Logger.info("Cleaned up #{length(expired_sessions)} expired sessions")
    end

    expired_sessions
  end
end
