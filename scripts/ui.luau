local store = require("scripts/store")
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

	local function renderUI()
		local bloomTxt = store.bloom:get() and "[ENABLED]" or "[DISABLED]"
		local atmoTxt = store.atmo:get() and "[ENABLED]" or "[DISABLED]"
		local colTxt = store.collision:get() and "[ENABLED]" or "[DISABLED]"

		text_label.Text = "CONTROLS\n\n"
			.. "[W A S D] Move\n[SPACE] Up\n[CTRL] Down\n[Q] First Person\n\n"
			.. "[B] Toggle Bloom "
			.. bloomTxt
			.. "\n"
			.. "[V] Toggle Atmosphere "
			.. atmoTxt
			.. "\n"
			.. "[R] Toggle Collision "
			.. colTxt
			.. "\n"
			.. "[Z] Cycle Collision Mode\n\n"
			.. "[F] Toggle UI"
	end

	store.bloom:on_change(renderUI)
	store.atmo:on_change(renderUI)
	store.collision:on_change(renderUI)

	store.uiVisible:on_change(function(isVisible)
		container.Visible = isVisible
	end)

	UserInputService.InputBegan:Connect(function(key: string)
		if key == "KeyF" then
			store.uiVisible:set(not store.uiVisible:get())
		end
	end)
end

return ui
