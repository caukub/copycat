const result = Log.hasLine("Ambiguous plugin name '{0}' for files 'plugins/{1}' and 'plugins/{2}' in 'plugins'", "AmbiguousPluginName");

if (result !== null) {
  return "AmbiguousPluginName";
}

>>>

[AmbiguousPluginName]
header = "There are 2 files for plugin {0}"
solutions = ["In the `/plugins/` directory, delete file {1} or {2} (if both are not the same, then delete the one with correct/newer version)"]
level = "warn"