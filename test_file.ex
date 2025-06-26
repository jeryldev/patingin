defmodule Test do
  def bad_function(input) do
    atom = String.to_atom(input)
    %User{name: atom}
  end
end
