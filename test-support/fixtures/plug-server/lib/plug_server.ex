defmodule PlugServer do
  use Plug.Router

  plug(:match)
  plug(:dispatch)

  get "/" do
    send_resp(conn, 200, "Hello from plug-server!\n")
  end

  match _ do
    send_resp(conn, 404, "Oops! Not Found\n")
  end

  def start_link(_) do
    port = String.to_integer(System.get_env("PORT") || "8080")
    IO.puts("plug-server starting on port #{port}")
    Plug.Cowboy.http(PlugServer, [], port: port)
  end

  def child_spec(opts) do
    %{
      id: PlugServer,
      start: {PlugServer, :start_link, [opts]},
      type: :worker,
      restart: :permanent,
      shutdown: 5000
    }
  end
end
