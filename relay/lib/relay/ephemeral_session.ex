defmodule Relay.EphemeralSession do
  @moduledoc """
  GenServer that manages ephemeral data sessions.

  Each session:
  - Stores data temporarily
  - Auto-expires after TTL
  - Destroys itself after data retrieval
  - Leaves no persistent traces
  """

  use GenServer
  require Logger

  alias Relay.SessionRegistry

  # Client API

  def start_link({session_id, data, ttl_seconds}) do
    GenServer.start_link(__MODULE__, {session_id, data, ttl_seconds},
      name: {:via, Registry, {SessionRegistry, session_id}}
    )
  end

  def get_and_destroy(pid) do
    GenServer.call(pid, :get_and_destroy)
  end

  # Server Callbacks

  @impl true
  def init({session_id, data, ttl_seconds}) do
    # Schedule automatic cleanup
    timer_ref = Process.send_after(self(), :expire, ttl_seconds * 1000)

    state = %{
      session_id: session_id,
      data: data,
      created_at: DateTime.utc_now(),
      expires_at: DateTime.add(DateTime.utc_now(), ttl_seconds, :second),
      timer_ref: timer_ref
    }

    Logger.info("Created ephemeral session #{session_id}, expires in #{ttl_seconds}s")

    {:ok, state}
  end

  @impl true
  def handle_call(:get_and_destroy, _from, state) do
    Logger.info("Session #{state.session_id} data retrieved, destroying session")

    # Cancel the timer since we're destroying manually
    Process.cancel_timer(state.timer_ref)

    # Return data and stop the process
    {:stop, :normal, {:ok, state.data}, state}
  end

  @impl true
  def handle_info(:expire, state) do
    Logger.info("Session #{state.session_id} expired, auto-destroying")
    {:stop, :normal, state}
  end

  @impl true
  def terminate(reason, state) do
    case reason do
      :normal ->
        Logger.info("Session #{state.session_id} terminated normally")
      other ->
        Logger.warn("Session #{state.session_id} terminated: #{inspect(other)}")
    end

    :ok
  end
end
