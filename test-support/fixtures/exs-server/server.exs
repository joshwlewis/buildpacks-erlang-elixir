# server.exs

defmodule SimpleHTTPServer do
  def start(port) do
    {:ok, listener} = :gen_tcp.listen(port, [:binary, packet: :raw, active: false, reuseaddr: true])
    IO.puts("exs-server listening on port #{port}...")
    loop(listener)
  end

  defp loop(listener) do
    {:ok, socket} = :gen_tcp.accept(listener)
    spawn(fn -> handle_request(socket) end)
    loop(listener)
  end

  defp handle_request(socket) do
    case :gen_tcp.recv(socket, 0) do
      {:ok, data} ->
        IO.puts("Received request:\n#{data}")
        response = build_response()
        :gen_tcp.send(socket, response)
        :gen_tcp.close(socket)
      {:error, _reason} ->
        :gen_tcp.close(socket)
    end
  end

  defp build_response() do
    """
    HTTP/1.1 200 OK\r
    Content-Type: text/plain\r
    Content-Length: 23\r
    \r
    Hello from exs-server!\n
    """
  end
end

port = System.get_env("PORT") |> String.to_integer()
SimpleHTTPServer.start(port)
