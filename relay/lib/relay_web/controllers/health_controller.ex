defmodule RelayWeb.HealthController do
  use RelayWeb, :controller

  def check(conn, _params) do
    json(conn, %{
      status: "healthy",
      timestamp: DateTime.utc_now(),
      service: "ProofZK Relay"
    })
  end
end
