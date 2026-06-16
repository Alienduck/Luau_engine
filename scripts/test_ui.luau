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

local up_right_image_container = Frame.new()
up_right_image_container.Parent = container
up_right_image_container.AnchorPoint = Vector2.new(1, 0)
up_right_image_container.Position = UDim2.fromScale(1, 0)
up_right_image_container.Size = UDim2.fromScale(.5, .5)
up_right_image_container.BackgroundTransparency = 0
up_right_image_container.BackgroundColor3 = Color3.fromRGB(200, 20, 20)

local up_right_image = ImageLabel.new()
up_right_image.Parent = up_right_image_container
up_right_image.AnchorPoint = Vector2.new(.5, .5)
up_right_image.Position = UDim2.fromScale(.5, .5)
up_right_image.Size = UDim2.fromOffset(100, 100)
up_right_image.BackgroundTransparency = 1
up_right_image.Image = "images/bevy_icon.png"

return container
