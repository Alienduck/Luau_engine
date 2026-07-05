local player = {}

-- Modules
local entity = require("./entity")

function player.Start()
	local subject = Part.new()
	subject.Name = "Subject"
	subject.Parent = workspace
	subject.Size = Vector3.one
	subject.Position = Vector3.new(0, 2, 0)
	subject.Transparency = 0
	subject.Material = Enum.Material.Neon
	subject.Shape = Enum.Shape.Capsule

	local s_collider = Collider.new()
	s_collider.Parent = subject

	local rb_subject = Rigidbody.new()
	rb_subject.Parent = subject
	rb_subject.GravityScale = 3

	local controller = CharacterController.new()
	controller.Parent = subject
	controller.WalkSpeed = 10
	controller.JumpPower = 20

	local keys_pressed = {}
	local entities: { entity.EntityModule } = {}

	UserInputService.InputBegan:Connect(function(key: string, _: string)
		keys_pressed[key] = true
		if key == "KeyQ" then
			Camera.FirstPerson = not Camera.FirstPerson
		end
		if key == "Space" then
			controller.Jump = true
		end
		if key == "KeyC" then
			controller.NoClip = not controller.NoClip
		end
		if key == "KeyX" then
			local entity = entity.new({ position = subject.Position })
			table.insert(entities, entity)
		end
		if key == "KeyE" then
			for i, entity in ipairs(entities) do
				entity.rb:ApplyImpulse(entity.rb.Velocity + Vector3.new(0, 5, 0))
			end
		end
		if key == "KeyJ" then
			subject.CastShadow = not subject.CastShadow
		end
	end)

	UserInputService.InputEnded:Connect(function(key: string, _: string)
		keys_pressed[key] = false
	end)

	RunService.RenderStepped:Connect(function(delta: number)
		local n = controller.NoClip
		local camLook = Camera.CFrame.LookVector
		local camRight = Camera.CFrame.RightVector
		camLook = Vector3.new(camLook.X, n and camLook.Y or 0, camLook.Z).Unit
		camRight = Vector3.new(camRight.X, n and camRight.Y or 0, camRight.Z).Unit

		local dir = Vector3.zero
		if keys_pressed["KeyW"] then
			dir = dir + camLook
		end
		if keys_pressed["KeyS"] then
			dir = dir - camLook
		end
		if keys_pressed["KeyA"] then
			dir = dir - camRight
		end
		if keys_pressed["KeyD"] then
			dir = dir + camRight
		end
		if keys_pressed["Space"] and n then
			dir = dir + Vector3.new(0, 1, 0)
		end

		if dir.Magnitude > 0 then
			dir = dir.Unit
		end

		controller.MoveDirection = dir
	end)

	local function random_color(): Color3
		return Color3.new(math.random(0, 255) / 255, math.random(0, 255) / 255, math.random(0, 255) / 255)
	end

	local subject_goal = random_color()
	RunService.RenderStepped:Connect(function(delta: number)
		subject.Color = subject.Color:Lerp(subject_goal, delta)
		if subject.Color == subject_goal then
			subject_goal = random_color()
		end
	end)

	Camera.Distance = 10
	return subject
end

return player
