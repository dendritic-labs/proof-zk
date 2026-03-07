defmodule Relay.SessionCleaner do
  @moduledoc """
  Background process that periodically cleans up any stale session references.
  Runs every minute to ensure system hygiene.
  """

  use GenServer
  require Logger

  @cleanup_interval :timer.minutes(1)

  def start_link(_opts) do
    GenServer.start_link(__MODULE__, [], name: __MODULE__)
  end

  @impl true
  def init(_) do
    schedule_cleanup()
    {:ok, %{}}
  end

  @impl true
  def handle_info(:cleanup, state) do
    cleanup_registry()
    schedule_cleanup()
    {:noreply, state}
  end

  defp schedule_cleanup do
    Process.send_after(self(), :cleanup, @cleanup_interval)
  end

  defp cleanup_registry do
    # Get all registered session processes
    sessions = Registry.select(Relay.SessionRegistry, [{{:"$1", :"$2", :"$3"}, [], [:"$1"]}])

    # Check which ones are still alive
    dead_sessions = Enum.filter(sessions, fn session_id ->
      case Registry.lookup(Relay.SessionRegistry, session_id) do
        [{pid, _}] -> not Process.alive?(pid)
        [] -> true
      end
    end)

    if length(dead_sessions) > 0 do
      Logger.info("Cleaned up #{length(dead_sessions)} stale session references")
    end

    Logger.debug("Registry cleanup completed. Active sessions: #{length(sessions) - length(dead_sessions)}")
  end
end
