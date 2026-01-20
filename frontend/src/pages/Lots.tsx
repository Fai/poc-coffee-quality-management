import { useTranslation } from 'react-i18next';
import { useQuery } from '@tanstack/react-query';
import { Plus, QrCode } from 'lucide-react';
import { lotsApi, Lot } from '../services/api';

const mockLots: Lot[] = [
  { id: '1', traceability_code: 'CQM-2024-DOI-0001', name: 'ล็อต A1 - Typica Natural', stage: 'green_bean', current_weight_kg: 45.5, created_at: '2024-01-15' },
  { id: '2', traceability_code: 'CQM-2024-DOI-0002', name: 'ล็อต A2 - Catimor Washed', stage: 'roasted_bean', current_weight_kg: 12.3, created_at: '2024-01-20' },
  { id: '3', traceability_code: 'CQM-2024-DOI-0003', name: 'ล็อต B1 - Geisha Honey', stage: 'cherry', current_weight_kg: 120.0, created_at: '2024-02-01' },
];

const stageColors: Record<string, string> = {
  cherry: 'bg-red-100 text-red-700',
  parchment: 'bg-yellow-100 text-yellow-700',
  green_bean: 'bg-green-100 text-green-700',
  roasted_bean: 'bg-amber-100 text-amber-700',
  sold: 'bg-gray-100 text-gray-700',
};

export default function Lots() {
  const { t } = useTranslation();
  const { data: lots = mockLots } = useQuery({
    queryKey: ['lots'],
    queryFn: lotsApi.list,
    retry: false,
    placeholderData: mockLots,
  });

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-coffee-800">{t('lots.title')}</h1>
        <button className="btn btn-primary flex items-center gap-2">
          <Plus size={20} />
          {t('lots.add')}
        </button>
      </div>

      <div className="card overflow-hidden">
        <div className="overflow-x-auto">
          <table className="w-full">
            <thead className="bg-coffee-50">
              <tr>
                <th className="px-4 py-3 text-left text-sm font-medium text-coffee-700">{t('lots.code')}</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-coffee-700">ชื่อ</th>
                <th className="px-4 py-3 text-left text-sm font-medium text-coffee-700">{t('lots.stage')}</th>
                <th className="px-4 py-3 text-right text-sm font-medium text-coffee-700">{t('lots.weight')}</th>
                <th className="px-4 py-3 text-center text-sm font-medium text-coffee-700">QR</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-coffee-100">
              {lots.map((lot) => (
                <tr key={lot.id} className="hover:bg-coffee-50 cursor-pointer">
                  <td className="px-4 py-3 font-mono text-sm">{lot.traceability_code}</td>
                  <td className="px-4 py-3">{lot.name}</td>
                  <td className="px-4 py-3">
                    <span className={`px-2 py-1 rounded-full text-xs ${stageColors[lot.stage] || 'bg-gray-100'}`}>
                      {t(`lots.stages.${lot.stage}`)}
                    </span>
                  </td>
                  <td className="px-4 py-3 text-right">{lot.current_weight_kg.toFixed(1)}</td>
                  <td className="px-4 py-3 text-center">
                    <button className="p-1 hover:bg-coffee-100 rounded">
                      <QrCode size={18} className="text-coffee-500" />
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </div>
    </div>
  );
}
