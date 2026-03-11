defmodule RelayWeb.ProofController do
  use RelayWeb, :controller

  def create_request(conn, _params) do
    json(conn, %{message: "Proof controller placeholder"})
  end

  def submit_response(conn, _params) do
    json(conn, %{message: "Proof controller placeholder"})
  end

  def get_request(conn, _params) do
    json(conn, %{message: "Proof controller placeholder"})
  end
end
