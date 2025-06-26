defmodule CleanCode do
  # This module has no violations - used for testing clean code scenarios
  
  def process_user_data(data) do
    case data do
      %{name: name, email: email} when is_binary(name) and is_binary(email) ->
        {:ok, %{name: name, email: email}}
      _ ->
        {:error, :invalid_data}
    end
  end
  
  def calculate_total(items) do
    Enum.reduce(items, 0, &(&1.amount + &2))
  end
  
  def format_response(result) do
    %{
      status: :success,
      data: result,
      timestamp: DateTime.utc_now()
    }
  end
end
