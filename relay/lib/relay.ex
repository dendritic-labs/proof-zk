defmodule Relay do
  @moduledoc """
  Ephemeral relay service for ProofZK system.

  This service creates temporary data sessions that automatically
  expire after successful proof transactions, ensuring no central
  data storage and maximum privacy.
  """

  alias Relay.{EphemeralSession, SessionRegistry}

  @doc """
  Create a new ephemeral session for proof data exchange.
  Returns a session ID that expires after the specified TTL.
  """
  def create_session(data, ttl_seconds \\ 300) do
    session_id = UUID.uuid4()

    case DynamicSupervisor.start_child(
      Relay.SessionSupervisor,
      {EphemeralSession, {session_id, data, ttl_seconds}}
    ) do
      {:ok, _pid} -> {:ok, session_id}
      {:error, reason} -> {:error, reason}
    end
  end

  @doc """
  Retrieve data from an ephemeral session.
  The session is destroyed after successful retrieval.
  """
  def get_session_data(session_id) do
    case Registry.lookup(SessionRegistry, session_id) do
      [{pid, _}] ->
        EphemeralSession.get_and_destroy(pid)
      [] ->
        {:error, :session_not_found}
    end
  end

  @doc """
  Check if a session exists without consuming it.
  """
  def session_exists?(session_id) do
    case Registry.lookup(SessionRegistry, session_id) do
      [{_pid, _}] -> true
      [] -> false
    end
  end

  @doc """
  Manually destroy a session before it expires.
  """
  def destroy_session(session_id) do
    case Registry.lookup(SessionRegistry, session_id) do
      [{pid, _}] ->
        GenServer.stop(pid)
        :ok
      [] ->
        {:error, :session_not_found}
    end
  end
end
