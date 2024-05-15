defmodule PathRouter do
  alias PathRouter.Native
  alias PathRouter.Router

  def new, do: Native.new() |> Router.wrap_resource()

  def add_route(%Router{resource: resource} = router, route, id) do
    Native.add_route(resource, route, id)
    router
  end

  def match_route(%Router{resource: resource}, path) do
    with {:ok, id, captures} <- Native.match_route(resource, path) do
      {:ok, id, format_captures(captures)}
    end
  end

  defp format_captures(captures) do
    case Map.new(captures) do
      %{"_remaining" => remaining} = map ->
        map
        |> Map.delete("_remaining")
        |> Map.merge(%{
          remaining: String.split(remaining, "/"),
          remaining_path: remaining
        })

      map ->
        map
    end
  end
end
