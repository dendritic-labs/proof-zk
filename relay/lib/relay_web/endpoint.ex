defmodule RelayWeb.Endpoint do
  use Phoenix.Endpoint, otp_app: :relay

  # The session will be stored in the cookie and signed,
  # this means its contents can be read but not tampered with.
  # Set :encryption_salt if you would also like to encrypt it.
  @session_options [
    store: :cookie,
    key: "_relay_key",
    signing_salt: "ephemeral_sessions",
    max_age: 86400
  ]

  socket "/live", Phoenix.LiveView.Socket, websocket: [connect_info: [session: @session_options]]

  # Serve at "/" the static files from "priv/static" directory.
  plug Plug.Static,
    at: "/",
    from: :relay,
    gzip: false,
    only: RelayWeb.static_paths()

  # Code reloading can be explicitly enabled under the
  # :code_reloader configuration of your endpoint.
  if code_reloading? do
    plug Phoenix.CodeReloader
  end

  plug Phoenix.LiveDashboard.RequestLogger,
    param_key: "request_logger",
    cookie_key: "request_logger"

  plug Plug.RequestId
  plug Plug.Telemetry, event_prefix: [:phoenix, :endpoint]

  plug Plug.Parsers,
    parsers: [:urlencoded, :multipart, :json],
    pass: ["*/*"],
    json_decoder: Phoenix.json_library()

  plug Plug.MethodOverride
  plug Plug.Head
  plug Plug.Session, @session_options

  # CORS configuration for cross-origin requests
  plug CORSPlug,
    origin: ["http://localhost:3000", "https://*.proofzk.com"],
    max_age: 86400,
    methods: ["GET", "POST", "PUT", "DELETE", "OPTIONS"]

  plug RelayWeb.Router
end
