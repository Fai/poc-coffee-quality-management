import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { Package, Layers, Scale, Award, Bell, Calendar, FileCheck } from 'lucide-react';
import { dashboardApi, DashboardMetrics } from '../services/api';

function StatCard({ icon: Icon, label, value, color }: { icon: React.ElementType; label: string; value: string | number; color: string }) {
  return (
    <div className="card flex items-center gap-4">
      <div className={`p-3 rounded-lg ${color}`}>
        <Icon className="text-white" size={24} />
      </div>
      <div>
        <p className="text-sm text-coffee-500">{label}</p>
        <p className="text-2xl font-bold text-coffee-800">{value}</p>
      </div>
    </div>
  );
}

// Mock data for demo
const mockMetrics: DashboardMetrics = {
  total_lots: 24,
  active_lots: 18,
  total_inventory_kg: 1250.5,
  avg_cupping_score: 84.5,
  pending_alerts: 3,
  recent_harvests: 5,
  expiring_certifications: 1,
};

export default function Dashboard() {
  const { t } = useTranslation();
  const { data: metrics = mockMetrics } = useQuery({
    queryKey: ['dashboard'],
    queryFn: dashboardApi.getMetrics,
    retry: false,
    placeholderData: mockMetrics,
  });

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-coffee-800">{t('dashboard.title')}</h1>

      <div className="grid grid-cols-2 lg:grid-cols-4 gap-4">
        <StatCard icon={Package} label={t('dashboard.totalLots')} value={metrics.total_lots} color="bg-coffee-500" />
        <StatCard icon={Layers} label={t('dashboard.activeLots')} value={metrics.active_lots} color="bg-coffee-400" />
        <StatCard icon={Scale} label={t('dashboard.inventory')} value={`${metrics.total_inventory_kg.toFixed(1)} ${t('common.kg')}`} color="bg-amber-500" />
        <StatCard icon={Award} label={t('dashboard.avgScore')} value={metrics.avg_cupping_score?.toFixed(1) ?? '-'} color="bg-green-500" />
      </div>

      <div className="grid lg:grid-cols-3 gap-4">
        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Calendar className="text-coffee-500" size={20} />
            <h2 className="font-semibold text-coffee-700">{t('dashboard.recentHarvests')}</h2>
          </div>
          <p className="text-3xl font-bold text-coffee-800">{metrics.recent_harvests}</p>
          <p className="text-sm text-coffee-500">ใน 7 วันที่ผ่านมา</p>
        </div>

        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <Bell className="text-orange-500" size={20} />
            <h2 className="font-semibold text-coffee-700">{t('dashboard.alerts')}</h2>
          </div>
          <p className="text-3xl font-bold text-orange-600">{metrics.pending_alerts}</p>
          <p className="text-sm text-coffee-500">รอดำเนินการ</p>
        </div>

        <div className="card">
          <div className="flex items-center gap-2 mb-4">
            <FileCheck className="text-red-500" size={20} />
            <h2 className="font-semibold text-coffee-700">{t('dashboard.expiringCerts')}</h2>
          </div>
          <p className="text-3xl font-bold text-red-600">{metrics.expiring_certifications}</p>
          <p className="text-sm text-coffee-500">ใน 90 วัน</p>
        </div>
      </div>
    </div>
  );
}
