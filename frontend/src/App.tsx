import { Routes, Route, Navigate } from 'react-router-dom';
import { useAuthStore } from './services/auth';
import Layout from './components/Layout';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Plots from './pages/Plots';
import Lots from './pages/Lots';
import Quality from './pages/Quality';
import Inventory from './pages/Inventory';
import DefectDetection from './pages/DefectDetection';

function PrivateRoute({ children }: { children: React.ReactNode }) {
  const isAuthenticated = useAuthStore((s) => s.isAuthenticated);
  return isAuthenticated ? <>{children}</> : <Navigate to="/login" replace />;
}

export default function App() {
  return (
    <Routes>
      <Route path="/login" element={<Login />} />
      <Route path="/" element={<PrivateRoute><Layout /></PrivateRoute>}>
        <Route index element={<Dashboard />} />
        <Route path="plots" element={<Plots />} />
        <Route path="lots" element={<Lots />} />
        <Route path="quality" element={<Quality />} />
        <Route path="inventory" element={<Inventory />} />
        <Route path="defect" element={<DefectDetection />} />
      </Route>
    </Routes>
  );
}
