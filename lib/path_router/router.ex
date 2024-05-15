defmodule PathRouter.Router do
  @moduledoc """
  Wrapper for the native PathRouter module.
  """
  defstruct resource: nil,
            reference: nil

  def wrap_resource(resource) do
    %__MODULE__{resource: resource, reference: make_ref()}
  end
end

defimpl Inspect, for: PathRouter.Router do
  import Inspect.Algebra

  def inspect(dict, opts) do
    concat(["#PathRouter.Router<", to_doc(dict.reference, opts), ">"])
  end
end
