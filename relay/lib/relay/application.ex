defmodule Relay.Application do
  @moduledoc """
  Ephemeral relay service for ProofZK

  Redesigned for high-performance session management:
  - ETS-based storage for millions of concurrent sessions
  - Single cleanup process for efficiency
  - Fault-tolerant supervision tree
  - Built-in observability and monitoring
  """

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      # High-performance ETS-based session storage
      Relay.SessionStore,

      # Phoenix endpoint for HTTP API
      RelayWeb.Endpoint
    ]

    opts = [strategy: :one_for_one, name: Relay.Supervisor]
    Supervisor.start_link(children, opts)
  end

  # Tell Phoenix to update the endpoint configuration
  # whenever the application is updated.
  @impl true
  def config_change(changed, _new, removed) do
    RelayWeb.Endpoint.config_change(changed, removed)
    :ok
  end
end
