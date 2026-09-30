-- Require a module only when it can be found on package.path.
-- Errors inside existing modules still surface normally.

local M = {}

-- Lua 5.1 does not have package.searchpath; provide a compatible fallback.
local searchpath = package.searchpath or function(name, path)
  local fname = name:gsub("%.", "/")
  for dir in (path .. ";"):gmatch("([^;]+);") do
    local file = dir:gsub("%?", fname)
    local f = io.open(file, "r")
    if f then f:close(); return file end
  end
  return nil
end

function M.module(module)
  if searchpath(module, package.path) then
    return require(module)
  end
end

return M
