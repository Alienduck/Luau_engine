local ui = {}

function ui.Start()
	local container = Frame.new()
	container.Position = UDim2.fromScale(0.02, 0.02)
	container.Size = UDim2.fromScale(0.25, 0.35)
	container.BackgroundTransparency = 0.5
	container.BackgroundColor3 = Color3.new(0, 0, 0)

	local text_label = TextLabel.new()
	text_label.Parent = container
	text_label.Position = UDim2.fromScale(0.05, 0.05)
	text_label.Size = UDim2.fromScale(0.9, 0.9)
	text_label.BackgroundTransparency = 1
	text_label.TextColor3 = Color3.new(1, 1, 1)
	text_label.Text =
		"CONTROLS\n\n[W A S D] Move\n[SPACE] Up\n[CTRL] Down\n[Q] First Person\n[B] Toggle Bloom\n[V] Toggle Atmosphere\n[F] Toggle UI"

	UserInputService.InputBegan:Connect(function(key: string, _: string)
		if key == "KeyF" then
			container.Visible = not container.Visible
		end
	end)
end

return ui
