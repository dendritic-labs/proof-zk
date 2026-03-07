defmodule RelayWeb.Router do
  use Phoenix.Router

  pipeline :api do
    plug :accepts, ["json"]
    plug :put_secure_browser_headers
  end

  scope "/api/v1", RelayWeb do
    pipe_through :api

    # Ephemeral session management
    post "/sessions", SessionController, :create
    get "/sessions/:session_id", SessionController, :get
    delete "/sessions/:session_id", SessionController, :destroy

    # Health check
    get "/health", HealthController, :check

    # Proof request handling
    post "/proof-requests", ProofController, :create_request
    post "/proof-responses", ProofController, :submit_response
    get "/proof-requests/:request_id", ProofController, :get_request
  end

  # Enables LiveDashboard only for development
  if Application.compile_env(:relay, :dev_routes) do
    import Phoenix.LiveDashboard.Router

    scope "/dev" do
      pipe_through [:fetch_session, :protect_from_forgery]

      live_dashboard "/dashboard", metrics: RelayWeb.Telemetry
    end
  end
end
