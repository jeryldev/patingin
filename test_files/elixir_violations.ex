defmodule ElixirViolations do
  # Dynamic atom creation (CRITICAL violation)
  def create_user_atom(name) do
    atom = String.to_atom(name)  # This is the violation
    %User{name: atom}
  end
  
  # Long parameter list (MAJOR violation)
  def complex_auth(email, password, token, device, ip, session, opts) do
    # Multiple violations in one function
    atom_key = String.to_atom("user_key")  # Another violation
    {:ok, atom_key}
  end
  
  # Good example (no violations)
  def safe_create_user(name) do
    case name do
      "admin" -> {:ok, :admin}
      "user" -> {:ok, :user}
      _ -> {:error, :invalid_user}
    end
  end
end
