defmodule PathRouter.Native do
  @moduledoc """
  Documentation for `MergePdf.Native`.
  """
  use Rustler,
    otp_app: :path_router,
    crate: "path_router"

  def new(), do: error()

  def add_string(_router, _string), do: error()

  def add_route(_router, _route, _id), do: error()

  def match_route(_router, _path), do: error()

  def get_strings(_router), do: error()

  defp error, do: :erlang.nif_error(:nif_not_loaded)
end
