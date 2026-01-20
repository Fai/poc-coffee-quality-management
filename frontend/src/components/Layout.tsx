import { Outlet, NavLink, useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { Home, Map, Package, Award, Warehouse, LogOut, Menu, X, Globe, ScanLine } from 'lucide-react';
import { useState } from 'react';
import { useAuthStore } from '../services/auth';
import clsx from 'clsx';

const navItems = [
  { path: '/', icon: Home, label: 'nav.dashboard' },
  { path: '/defect', icon: ScanLine, label: 'nav.defect' },
  { path: '/plots', icon: Map, label: 'nav.plots' },
  { path: '/lots', icon: Package, label: 'nav.lots' },
  { path: '/quality', icon: Award, label: 'nav.quality' },
  { path: '/inventory', icon: Warehouse, label: 'nav.inventory' },
];

export default function Layout() {
  const { t, i18n } = useTranslation();
  const [menuOpen, setMenuOpen] = useState(false);
  const logout = useAuthStore((s) => s.logout);
  const navigate = useNavigate();

  const handleLogout = () => { logout(); navigate('/login'); };
  const toggleLang = () => i18n.changeLanguage(i18n.language === 'th' ? 'en' : 'th');

  return (
    <div className="min-h-screen flex flex-col">
      {/* Header */}
      <header className="bg-coffee-600 text-white px-4 py-3 flex items-center justify-between sticky top-0 z-50">
        <button onClick={() => setMenuOpen(!menuOpen)} className="lg:hidden p-1">
          {menuOpen ? <X size={24} /> : <Menu size={24} />}
        </button>
        <h1 className="text-lg font-semibold">{t('app.name')}</h1>
        <div className="flex items-center gap-2">
          <button onClick={toggleLang} className="p-2 hover:bg-coffee-700 rounded-lg" title="Toggle language">
            <Globe size={20} />
          </button>
        </div>
      </header>

      <div className="flex flex-1">
        {/* Sidebar */}
        <nav className={clsx(
          'fixed lg:static inset-y-0 left-0 z-40 w-64 bg-white border-r border-coffee-100 transform transition-transform lg:translate-x-0',
          menuOpen ? 'translate-x-0' : '-translate-x-full'
        )}>
          <div className="p-4 pt-16 lg:pt-4 space-y-1">
            {navItems.map(({ path, icon: Icon, label }) => (
              <NavLink
                key={path}
                to={path}
                onClick={() => setMenuOpen(false)}
                className={({ isActive }) => clsx(
                  'flex items-center gap-3 px-3 py-2 rounded-lg transition-colors',
                  isActive ? 'bg-coffee-100 text-coffee-700' : 'text-coffee-600 hover:bg-coffee-50'
                )}
              >
                <Icon size={20} />
                <span>{t(label)}</span>
              </NavLink>
            ))}
            <hr className="my-4 border-coffee-100" />
            <button onClick={handleLogout} className="flex items-center gap-3 px-3 py-2 rounded-lg text-coffee-600 hover:bg-coffee-50 w-full">
              <LogOut size={20} />
              <span>{t('nav.logout')}</span>
            </button>
          </div>
        </nav>

        {/* Overlay */}
        {menuOpen && <div className="fixed inset-0 bg-black/50 z-30 lg:hidden" onClick={() => setMenuOpen(false)} />}

        {/* Main content */}
        <main className="flex-1 p-4 lg:p-6 overflow-auto">
          <Outlet />
        </main>
      </div>
    </div>
  );
}
