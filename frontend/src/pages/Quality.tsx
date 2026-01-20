import { useTranslation } from 'react-i18next';
import { LineChart, Line, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';

const mockTrendData = [
  { month: 'ม.ค.', score: 82 },
  { month: 'ก.พ.', score: 84 },
  { month: 'มี.ค.', score: 83 },
  { month: 'เม.ย.', score: 85 },
  { month: 'พ.ค.', score: 86 },
  { month: 'มิ.ย.', score: 84 },
];

const mockGradings = [
  { id: '1', lot: 'CQM-2024-DOI-0001', grade: 'Specialty', defects: 3, score: 86 },
  { id: '2', lot: 'CQM-2024-DOI-0002', grade: 'Premium', defects: 7, score: 82 },
  { id: '3', lot: 'CQM-2024-DOI-0003', grade: 'Specialty', defects: 2, score: 88 },
];

export default function Quality() {
  const { t } = useTranslation();

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-coffee-800">{t('quality.title')}</h1>

      <div className="grid lg:grid-cols-2 gap-6">
        {/* Quality Trend Chart */}
        <div className="card">
          <h2 className="font-semibold text-coffee-700 mb-4">แนวโน้มคะแนนคัปปิ้ง</h2>
          <div className="h-64">
            <ResponsiveContainer width="100%" height="100%">
              <LineChart data={mockTrendData}>
                <CartesianGrid strokeDasharray="3 3" stroke="#E8CBA7" />
                <XAxis dataKey="month" stroke="#6B4423" />
                <YAxis domain={[75, 95]} stroke="#6B4423" />
                <Tooltip />
                <Line type="monotone" dataKey="score" stroke="#6B4423" strokeWidth={2} dot={{ fill: '#6B4423' }} />
              </LineChart>
            </ResponsiveContainer>
          </div>
        </div>

        {/* Recent Gradings */}
        <div className="card">
          <h2 className="font-semibold text-coffee-700 mb-4">การเกรดล่าสุด</h2>
          <div className="space-y-3">
            {mockGradings.map((g) => (
              <div key={g.id} className="flex items-center justify-between p-3 bg-coffee-50 rounded-lg">
                <div>
                  <p className="font-mono text-sm text-coffee-600">{g.lot}</p>
                  <p className="text-sm text-coffee-500">{g.defects} defects</p>
                </div>
                <div className="text-right">
                  <span className={`px-2 py-1 rounded-full text-xs ${g.grade === 'Specialty' ? 'bg-green-100 text-green-700' : 'bg-amber-100 text-amber-700'}`}>
                    {g.grade}
                  </span>
                  <p className="text-lg font-bold text-coffee-800 mt-1">{g.score}</p>
                </div>
              </div>
            ))}
          </div>
        </div>
      </div>
    </div>
  );
}
