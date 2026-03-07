defmodule RelayWeb.SessionController do
  use Phoenix.Controller

  action_fallback RelayWeb.FallbackController

  @doc """
  Create a new ephemeral session with TTL
  """
  def create(conn, %{"data" => data} = params) do
    ttl = Map.get(params, "ttl", 300) # Default 5 minutes

    case Relay.create_session(data, ttl) do
      {:ok, session_id} ->
        conn
        |> put_status(:created)
        |> json(%{
          session_id: session_id,
          expires_in: ttl,
          message: "Ephemeral session created"
        })

      {:error, reason} ->
        conn
        |> put_status(:internal_server_error)
        |> json(%{error: "Failed to create session", reason: reason})
    end
  end

  @doc """
  Retrieve and destroy session data
  """
  def get(conn, %{"session_id" => session_id}) do
    case Relay.get_session_data(session_id) do
      {:ok, data} ->
        conn
        |> json(%{
          data: data,
          message: "Session data retrieved and destroyed"
        })

      {:error, :session_not_found} ->
        conn
        |> put_status(:not_found)
        |> json(%{error: "Session not found or expired"})
    end
  end

  @doc """
  Manually destroy a session
  """
  def destroy(conn, %{"session_id" => session_id}) do
    case Relay.destroy_session(session_id) do
      :ok ->
        conn
        |> json(%{message: "Session destroyed"})

      {:error, :session_not_found} ->
        conn
        |> put_status(:not_found)
        |> json(%{error: "Session not found"})
    end
  end
end
