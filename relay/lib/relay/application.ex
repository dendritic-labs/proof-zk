defmodule Relay.Application do
  @moduledoc """
  Ephemeral relay service for ProofZK
  Handles temporary data sessions that disappear after transactions
  """

  use Application

  @impl true
  def start(_type, _args) do
    children = [
      # Start the registry for dynamic session processes
      {Registry, keys: :unique, name: Relay.SessionRegistry},

      # Start the session supervisor
      {DynamicSupervisor, name: Relay.SessionSupervisor, strategy: :one_for_one},

      # Start the cleanup agent for expired sessions
      Relay.SessionCleaner,

      # Start the Phoenix endpoint
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
