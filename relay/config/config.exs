import Config

# We don't run a server during test. If one is required,
# you can enable the server option below.
config :relay, RelayWeb.Endpoint,
  http: [ip: {127, 0, 0, 1}, port: 4000],
  secret_key_base: "ephemeral_relay_secret_key_base_for_development_only",
  live_view: [signing_salt: "ephemeral_relay"],
  server: true

# Print only warnings and errors during test
config :logger, level: :info

# Initialize plugs at runtime for faster test compilation
config :phoenix, :plug_init_mode, :runtime
