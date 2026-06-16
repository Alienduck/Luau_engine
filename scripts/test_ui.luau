local container = Frame.new()
container.Position = UDim2.fromScale(0, 0)
container.Size = UDim2.fromScale(1, 1)
container.BackgroundTransparency = 0
container.BackgroundColor3 = Color3.new(0, 0, 0)

local up_left_text = TextLabel.new()
up_left_text.Parent = container
up_left_text.Position = UDim2.fromScale(0, 0)
up_left_text.Size = UDim2.fromScale(0.5, 0.5)
up_left_text.Text = "UP LEFT TEXT"
up_left_text.TextColor3 = Color3.new(1, 1, 1)
up_left_text.BackgroundColor3 = Color3.fromRGB(20, 20, 200)
up_left_text.BackgroundTransparency = 0.5

return container
