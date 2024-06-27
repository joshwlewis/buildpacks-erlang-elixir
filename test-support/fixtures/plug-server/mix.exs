defmodule PlugServer.MixProject do
  use Mix.Project

  def project do
    [
      app: :plug_server,
      version: "0.1.0",
      elixir: "~> 1.17",
      start_permanent: Mix.env() == :prod,
      deps: deps()
    ]
  end

  def application do
    [
      mod: {PlugServer.Application, []},
      extra_applications: [:logger]
    ]
  end

  defp deps do
    [
      {:plug_cowboy, "~> 2.5"},
      {:plug, "~> 1.11"}
    ]
  end
end
