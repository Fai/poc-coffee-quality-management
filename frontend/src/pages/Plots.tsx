import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { Plus, MapPin, Mountain } from 'lucide-react';
import { plotsApi, Plot } from '../services/api';

// Mock data
const mockPlots: Plot[] = [
  { id: '1', name: 'แปลง A - ดอยช้าง', area_rai: 5.5, altitude_meters: 1200, shade_coverage_percent: 40, varieties: [{ variety: 'Typica', planting_date: '2020-01-15' }] },
  { id: '2', name: 'แปลง B - ดอยสะเก็ด', area_rai: 3.2, altitude_meters: 1100, shade_coverage_percent: 30, varieties: [{ variety: 'Catimor', planting_date: '2019-06-01' }] },
  { id: '3', name: 'แปลง C - แม่แจ่ม', area_rai: 8.0, altitude_meters: 1350, shade_coverage_percent: 50, varieties: [{ variety: 'Geisha', planting_date: '2021-03-10' }] },
];

export default function Plots() {
  const { t } = useTranslation();
  const { data: plots = mockPlots } = useQuery({
    queryKey: ['plots'],
    queryFn: plotsApi.list,
    retry: false,
    placeholderData: mockPlots,
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-coffee-800">{t('plots.title')}</h1>
        <button className="btn btn-primary flex items-center gap-2">
          <Plus size={20} />
          {t('plots.add')}
        </button>
      </div>

      <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-4">
        {plots.map((plot) => (
          <div key={plot.id} className="card hover:shadow-md transition-shadow cursor-pointer">
            <h3 className="font-semibold text-coffee-800 mb-2">{plot.name}</h3>
            <div className="space-y-2 text-sm text-coffee-600">
              <div className="flex items-center gap-2">
                <MapPin size={16} />
                <span>{plot.area_rai} {t('plots.area')}</span>
              </div>
              {plot.altitude_meters && (
                <div className="flex items-center gap-2">
                  <Mountain size={16} />
                  <span>{plot.altitude_meters} {t('plots.altitude')}</span>
                </div>
              )}
              <div className="flex flex-wrap gap-1 mt-2">
                {plot.varieties.map((v, i) => (
                  <span key={i} className="px-2 py-1 bg-coffee-100 text-coffee-700 rounded-full text-xs">
                    {v.variety}
                  </span>
                ))}
              </div>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
