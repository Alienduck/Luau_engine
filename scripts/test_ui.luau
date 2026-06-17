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

local down_left_text_button = TextButton.new()
down_left_text_button.Parent = container
down_left_text_button.AnchorPoint = Vector2.new(0, 1)
down_left_text_button.Position = UDim2.fromScale(0, 1)
down_left_text_button.Size = UDim2.fromScale(.5, .5)
down_left_text_button.BackgroundColor3 = Color3.fromRGB(20, 200, 20)
down_left_text_button.Text = "DOWN LEFT TEXT BUTTON (CLICK ME)"
down_left_text_button.TextColor3 = Color3.new(0, 0, 0)
down_left_text_button.MouseEnter:Connect(function()
    print("mouse entenring on [DOWN LEFT TEXT BUTTON]")
end)
down_left_text_button.MouseLeave:Connect(function()
    print("mouse leaving on [DOWN LEFT TEXT BUTTON]")
end)
down_left_text_button.MouseButton1Click:Connect(function()
    print("mouse click on [DOWN LEFT TEXT BUTTON]")
end)

local down_right_image_container = Frame.new()
down_right_image_container.Parent = container
down_right_image_container.AnchorPoint = Vector2.one
down_right_image_container.Position = UDim2.fromScale(1, 1)
down_right_image_container.Size = UDim2.fromScale(.5, .5)
down_right_image_container.BackgroundColor3 = Color3.fromRGB(200, 20, 200)

local down_right_image_button = ImageButton.new()
down_right_image_button.Parent = down_right_image_container
down_right_image_button.Image = "images/bevy_icon.png"
down_right_image_button.AnchorPoint = Vector2.new(.5, .5)
down_right_image_button.Position = UDim2.fromScale(.5, .5)
down_right_image_button.Size = UDim2.fromOffset(100, 100)
down_right_image_button.BackgroundTransparency = 1
down_right_image_button.MouseEnter:Connect(function()
    print("mouse entenring on [DOWN RIGHT IMAGE BUTTON]")
    local goal = { Size = UDim2.fromOffset(110, 110)}
    TweenService:Create(
        down_right_image_button,
        TweenInfo.new(0.1),
        goal
    ):Play()
end)
down_right_image_button.MouseLeave:Connect(function()
    print("mouse leaving on [DOWN RIGHT IMAGE BUTTON]")
    local goal = { Size = UDim2.fromOffset(100, 100)}
    TweenService:Create(
        down_right_image_button,
        TweenInfo.new(0.1),
        goal
    ):Play()
end)
down_right_image_button.MouseButton1Click:Connect(function()
    print("mouse click on [DOWN RIGHT IMAGE BUTTON]")
    local goal = { Size = Vector2.new(.1, .1)}
    TweenService:Create(
        down_right_image_button,
        TweenInfo.new(0.1),
        goal
    ):Play()
end)


return container
