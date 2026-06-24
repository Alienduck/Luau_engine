local env = {}

function env.Start()
	Lighting.Brightness = 10
	Lighting.Ambient = Color3.fromRGB(0, 0, 100)
	Lighting.GlobalShadows = true
	Lighting.ClockTime = 0
	Lighting.FogStart = 20
	Lighting.FogEnd = 500

	RunService.RenderStepped:Connect(function(delta: number)
		Lighting.ClockTime += (delta / 3600) * 2.4
	end)

	local sky = Sky.new()
	sky.CubemapPath = "skybox/sky_skybox.ktx2"
	sky.Parent = Lighting

	local bloom = BloomEffect.new()
	bloom.Intensity = 1
	bloom.Size = 24.0
	bloom.Threshold = 1
	bloom.Parent = Lighting

	local bloom_start = TweenService:Create(bloom, TweenInfo.new(3), { Size = 50 })
	local bloom_end = TweenService:Create(bloom, TweenInfo.new(3), { Size = 0 })
	bloom_start.Completed:Connect(function()
		bloom_end:Play()
	end)
	bloom_end.Completed:Connect(function()
		bloom_start:Play()
	end)
	bloom_start:Play()

	local atmosphere = Atmosphere.new()
	atmosphere.Parent = Lighting
	atmosphere.Density = 0

	local atmo_start = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 1 })
	local atmo_end = TweenService:Create(atmosphere, TweenInfo.new(5), { Density = 0 })
	atmo_start.Completed:Connect(function()
		atmo_end:Play()
	end)
	atmo_end.Completed:Connect(function()
		atmo_start:Play()
	end)
	atmo_start:Play()

	local bloom_enabled, atmo_enabled = true, true

	UserInputService.InputBegan:Connect(function(key: string, _: string)
		if key == "KeyB" then
			bloom_enabled = not bloom_enabled
			if bloom_enabled then
				bloom_start:Play()
			else
				bloom_start:Pause()
				bloom_end:Pause()
				bloom.Intensity = 0
			end
		elseif key == "KeyV" then
			atmo_enabled = not atmo_enabled
			if atmo_enabled then
				atmo_start:Play()
			else
				atmo_start:Pause()
				atmo_end:Pause()
				atmosphere.Density = 0
			end
		elseif key == "KeyR" then
			CollisionRender.Enable = not CollisionRender.Enable
		elseif key == "KeyZ" then
			CollisionRender.DebugMode = (CollisionRender.DebugMode + 1) % (Enum.ColliderRenderMode.ColliderAABBS + 1)
			print("Debug Mode passé à :", CollisionRender.DebugMode)
		end
	end)

	local rain = require("scripts/rain")
	rain.Start()

	local mesh_part = MeshPart.new()
	mesh_part.Name = "map"
	mesh_part.Parent = workspace
	mesh_part.CollisionFidelity = Enum.CollisionFidelity.PreciseConvexDecomposition
	mesh_part.MeshId = 2 == 1 and "models/chicken_idk_map/scene.gltf#Scene0"
		or "models/stylized_classroom/scene.gltf#Scene0"
	mesh_part.Position = Vector3.zero

	local cl = Collider.new()
	cl.Parent = mesh_part
	local rb = Rigidbody.new()
	rb.Anchored = true
	rb.Parent = mesh_part

	return mesh_part
end

return env
