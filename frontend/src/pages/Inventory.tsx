import { useTranslation } from 'react-i18next';
import { BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const mockInventory = [
  { stage: 'เชอร์รี่', weight: 450 },
  { stage: 'กะลา', weight: 280 },
  { stage: 'สารกาแฟ', weight: 520 },
  { stage: 'เมล็ดคั่ว', weight: 180 },
];

const mockTransactions = [
  { id: '1', lot: 'CQM-2024-DOI-0001', type: 'harvest_in', qty: 120, date: '2024-02-01' },
  { id: '2', lot: 'CQM-2024-DOI-0001', type: 'processing_out', qty: -120, date: '2024-02-05' },
  { id: '3', lot: 'CQM-2024-DOI-0001', type: 'processing_in', qty: 45, date: '2024-02-15' },
  { id: '4', lot: 'CQM-2024-DOI-0002', type: 'sale', qty: -10, date: '2024-02-18' },
];

const typeLabels: Record<string, string> = {
  harvest_in: 'เก็บเกี่ยว',
  processing_out: 'ส่งแปรรูป',
  processing_in: 'รับจากแปรรูป',
  roasting_out: 'ส่งคั่ว',
  roasting_in: 'รับจากคั่ว',
  sale: 'ขาย',
  purchase: 'ซื้อ',
};

export default function Inventory() {
  const { t } = useTranslation();

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-coffee-800">{t('inventory.title')}</h1>

      <div className="grid lg:grid-cols-2 gap-6">
        {/* Inventory by Stage */}
        <div className="card">
          <h2 className="font-semibold text-coffee-700 mb-4">{t('inventory.balance')}</h2>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <BarChart data={mockInventory}>
                <CartesianGrid strokeDasharray="3 3" stroke="#E8CBA7" />
                <XAxis dataKey="stage" stroke="#6B4423" />
                <YAxis stroke="#6B4423" />
                <Tooltip />
                <Bar dataKey="weight" fill="#8B5A2B" radius={[4, 4, 0, 0]} />
              </BarChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Recent Transactions */}
        <div className="card">
          <h2 className="font-semibold text-coffee-700 mb-4">{t('inventory.transactions')}</h2>
          <div className="space-y-2">
            {mockTransactions.map((tx) => (
              <div key={tx.id} className="flex items-center justify-between p-3 bg-coffee-50 rounded-lg">
                <div>
                  <p className="font-mono text-sm text-coffee-600">{tx.lot}</p>
                  <p className="text-sm text-coffee-500">{typeLabels[tx.type] || tx.type}</p>
                </div>
                <div className="text-right">
                  <p className={`font-bold ${tx.qty > 0 ? 'text-green-600' : 'text-red-600'}`}>
                    {tx.qty > 0 ? '+' : ''}{tx.qty} {t('common.kg')}
                  </p>
                  <p className="text-xs text-coffee-400">{tx.date}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
