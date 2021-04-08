defmodule UmeWeb.MainController do
  use UmeWeb, :controller

  def index(conn, _params) do
    json(conn, %{uwu: "hi"})
  end

end
