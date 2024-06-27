-module(hello).
-export([main/1]).

main([Ports | _]) ->
  Port = erlang:list_to_integer(Ports),
  start(Port),
  io:format("escript-server listening on port ~p.~n", [Port]),
  receive
    stop -> ok
  end.

start(Port) ->
    spawn(fun () -> {ok, Sock} = gen_tcp:listen(Port, [{active, false}]),
                    loop(Sock) end).

loop(Sock) ->
    {ok, Conn} = gen_tcp:accept(Sock),
    Handler = spawn(fun () -> handle(Conn) end),
    gen_tcp:controlling_process(Conn, Handler),
    loop(Sock).

handle(Conn) ->
    gen_tcp:send(Conn, response("Hello from escript-server!\n")),
    gen_tcp:close(Conn).

response(Str) ->
    B = iolist_to_binary(Str),
    iolist_to_binary(
      io_lib:fwrite(
         "HTTP/1.0 200 OK\nContent-Type: text/html\nContent-Length: ~p\n\n~s",
         [size(B), B])).
