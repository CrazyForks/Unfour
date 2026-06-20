import { DesktopApp } from "@unfour/app-shell";

// The full shell composition now lives in @unfour/app-shell (DesktopApp).
// apps/desktop is a thin entry that mounts it; the Pro edition mounts the
// same DesktopApp from the unfour submodule with zero code duplication.
function App() {
  return <DesktopApp />;
}

export default App;
