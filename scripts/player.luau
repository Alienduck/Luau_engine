local player = {}

-- Modules
local entity = require("./entity")

function player.Start()
	local subject = Part.new()
	subject.Name = "Subject"
	subject.Parent = workspace
	subject.Size = Vector3.one
	subject.Position = Vector3.new(0, 10, 0)
	subject.Transparency = 0
	subject.Material = Enum.Material.Neon
	subject.Shape = Enum.Shape.Capsule

	PhysicsService:RegisterCollisionGroup("Player")
	PhysicsService:CollisionGroupSetCollidable("Default", "Player", true)
	local s_collider = Collider.new()
	s_collider.Parent = subject
	s_collider.CollisionGroup = "Player"

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
		if key == "Tab" then
			print(rb_subject.Velocity)
		end
		if key == "Left" then
			local origin = Camera.CFrame.Position
			local raycast_params = RaycastParams.new()

			if Camera.FirstPerson then
				raycast_params.FilterDescendantInstances = { subject }
				raycast_params.FilterType = Enum.RaycastFilterType.Exclude
			end

			local result = workspace:Raycast(origin, Camera.CFrame.LookVector * 100, raycast_params)
			if not result then
				return
			end
			print(result)

			local dist = (result.Position - origin).Magnitude

			local ray_part = Part.new()
			ray_part.Size = Vector3.new(0.05, 0.05, dist)
			ray_part.CFrame = CFrame.lookAt(origin, result.Position) * CFrame.new(0, 0, -dist / 2)
			ray_part.Color = Color3.new(1, 0, 0)
			ray_part.Parent = workspace

			local norm_len = 2.0
			local norm_part = Part.new()
			norm_part.Parent = workspace
			norm_part.Size = Vector3.new(0.05, 0.05, norm_len)
			norm_part.CFrame = CFrame.lookAt(result.Position, result.Position + result.Normal)
				* CFrame.new(0, 0, -norm_len / 2)
			norm_part.Color = Color3.new(0, 1, 0)

			task.delay(1, function()
				ray_part:Destroy()
				norm_part:Destroy()
			end)
		end
	end)

	UserInputService.InputEnded:Connect(function(key: string, _: string)
		keys_pressed[key] = false
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
