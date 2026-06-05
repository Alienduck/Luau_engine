local module = {}

export type TestModuleType = { test: () -> () }

function module.test()
    print("Hello World!")
end

return module
