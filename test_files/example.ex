defmodule Example do
  # Example with dynamic atom creation (anti-pattern)
  def process_status(status_string) do
    # This is bad - creates atoms dynamically
    atom = String.to_atom(status_string)
    
    case atom do
      :ok -> "Success"
      :error -> "Failed"
      _ -> "Unknown"
    end
  end

  # Example with long parameter list (anti-pattern)
  def create_user(name, email, password, age, address, phone, country, zip_code) do
    %{
      name: name,
      email: email,
      password: password,
      age: age,
      address: address,
      phone: phone,
      country: country,
      zip_code: zip_code
    }
  end

  # Good example
  def process_status_safe(status_string) do
    case status_string do
      "ok" -> :ok
      "error" -> :error
      _ -> :unknown
    end
  end
end