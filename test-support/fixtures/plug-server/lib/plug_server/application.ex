defmodule PlugServer.Application do
  use Application

  def start(_type, _args) do
    opts = [strategy: :one_for_one, name: PlugServer.Supervisor]
    Supervisor.start_link([PlugServer], opts)
  end
end
