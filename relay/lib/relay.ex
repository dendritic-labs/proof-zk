defmodule Relay do
  @moduledoc """
  Ephemeral relay service for ProofZK system.

  This service creates temporary data sessions that automatically
  expire after successful proof transactions, ensuring no central
  data storage and maximum privacy.

  Redesigned for high performance using ETS storage:
  - Handles millions of concurrent sessions
  - Microsecond lookup performance
  - Atomic operations for data consistency
  - Single cleanup process for efficiency
  """

  alias Relay.SessionStore

  @doc """
  Create a new ephemeral session for proof data exchange.
  Returns {:ok, session_id} or {:error, reason}.

  ## Examples

      iex> {:ok, session_id} = Relay.create_session(%{"proof" => "age_over_21"}, 300)
      iex> is_binary(session_id)
      true

      iex> Relay.create_session(%{}, 0)
      {:error, :invalid_ttl}
  """
  def create_session(data, ttl_seconds \\ 300)

  def create_session(data, ttl_seconds) when is_integer(ttl_seconds) and ttl_seconds > 0 do
    SessionStore.create_session(data, ttl_seconds)
  end

  def create_session(_data, _ttl_seconds) do
    {:error, :invalid_ttl}
  end

  @doc """
  Retrieve data from an ephemeral session.
  The session is destroyed after successful retrieval.

  ## Examples

      iex> {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 60)
      iex> {:ok, data} = Relay.get_session_data(session_id)
      iex> data
      %{"test" => "data"}

      iex> Relay.get_session_data("550e8400-e29b-41d4-a716-446655440000")
      {:error, :not_found}

      iex> Relay.get_session_data("invalid-uuid")
      {:error, :invalid_session_id}
  """
  def get_session_data(session_id) when is_binary(session_id) do
    if valid_uuid?(session_id) do
      SessionStore.get_and_destroy_session(session_id)
    else
      {:error, :invalid_session_id}
    end
  end

  def get_session_data(_session_id) do
    {:error, :invalid_session_id}
  end

  @doc """
  Check if a session exists without consuming it.

  ## Examples

      iex> {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 60)
      iex> Relay.session_exists?(session_id)
      true

      iex> Relay.session_exists?("nonexistent")
      false
  """
  def session_exists?(session_id) when is_binary(session_id) do
    if valid_uuid?(session_id) do
      SessionStore.session_exists?(session_id)
    else
      false
    end
  end

  def session_exists?(_session_id), do: false

  @doc """
  Manually destroy a session before it expires.

  ## Examples

      iex> {:ok, session_id} = Relay.create_session(%{"test" => "data"}, 60)
      iex> Relay.destroy_session(session_id)
      :ok

      iex> Relay.destroy_session("550e8400-e29b-41d4-a716-446655440000")
      {:error, :not_found}

      iex> Relay.destroy_session("invalid-uuid")
      {:error, :invalid_session_id}
  """
  def destroy_session(session_id) when is_binary(session_id) do
    if valid_uuid?(session_id) do
      SessionStore.destroy_session(session_id)
    else
      {:error, :invalid_session_id}
    end
  end

  def destroy_session(_session_id) do
    {:error, :invalid_session_id}
  end

  @doc """
  Get statistics about current sessions and system performance.

  Returns a map with session and memory statistics.

  ## Examples

      iex> stats = Relay.stats()
      iex> is_map(stats)
      true
      iex> Map.has_key?(stats, :total_sessions)
      true
  """
  def stats do
    SessionStore.stats()
  end

  # Private helper to validate UUID format
  defp valid_uuid?(session_id) do
    case String.length(session_id) do
      36 -> String.match?(session_id, ~r/^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/i)
      _ -> false
    end
  end
end
