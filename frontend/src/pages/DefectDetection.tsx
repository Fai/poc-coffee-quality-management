import { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Camera, Upload, Loader2, AlertCircle, RefreshCw } from 'lucide-react';
import { defectApi, DefectDetectionResult } from '../services/api';

const MAX_RETRIES = 3;
const RETRY_DELAY = 2000;

const DEFECT_COLORS: Record<string, string> = {
  black: 'bg-gray-800 text-white',
  broken: 'bg-amber-600 text-white',
  foreign: 'bg-purple-600 text-white',
  fraghusk: 'bg-yellow-500 text-black',
  green: 'bg-green-600 text-white',
  husk: 'bg-orange-500 text-white',
  immature: 'bg-lime-500 text-black',
  infested: 'bg-red-700 text-white',
  sour: 'bg-pink-500 text-white',
};

export default function DefectDetection() {
  const { t } = useTranslation();
  const [image, setImage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [loadingMsg, setLoadingMsg] = useState('');
  const [result, setResult] = useState<DefectDetectionResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [useYolo, setUseYolo] = useState(true);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const cameraInputRef = useRef<HTMLInputElement>(null);

  const detectWithRetry = async (base64: string, attempt = 1): Promise<DefectDetectionResult> => {
    try {
      setLoadingMsg(attempt > 1 ? t('defect.retrying', { attempt }) : t('defect.analyzing'));
      return useYolo ? await defectApi.detectYolo(base64) : await defectApi.detect(base64);
    } catch (err) {
      const isTimeout = err instanceof Error && (err.message.includes('timeout') || err.message.includes('Internal Server Error'));
      if (isTimeout && attempt < MAX_RETRIES) {
        setLoadingMsg(t('defect.coldStart'));
        await new Promise(r => setTimeout(r, RETRY_DELAY));
        return detectWithRetry(base64, attempt + 1);
      }
      throw err;
    }
  };

  const handleFile = async (file: File) => {
    setError(null);
    setResult(null);
    
    const reader = new FileReader();
    reader.onload = async (e) => {
      const base64 = (e.target?.result as string).split(',')[1];
      setImage(e.target?.result as string);
      
      setLoading(true);
      try {
        const res = await detectWithRetry(base64);
        setResult(res);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Detection failed');
      } finally {
        setLoading(false);
        setLoadingMsg('');
      }
    };
    reader.readAsDataURL(file);
  };

  const handleFileChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    const file = e.target.files?.[0];
    if (file) handleFile(file);
  };

  const reset = () => {
    setImage(null);
    setResult(null);
    setError(null);
  };

  const isYoloResult = result?.detection.total_defects !== undefined;
  const totalDefects = result?.detection.total_defects ?? 0;
  const defectCounts = result?.detection.defect_counts ?? {};

  return (
    <div className="space-y-6">
      <div className="flex items-center justify-between">
        <h1 className="text-2xl font-bold text-coffee-800">{t('defect.title')}</h1>
        <label className="flex items-center gap-2 text-sm">
          <input type="checkbox" checked={useYolo} onChange={e => setUseYolo(e.target.checked)} className="rounded" />
          <span className="text-coffee-600">{t('defect.countMode')}</span>
        </label>
      </div>

      {!image ? (
        <div className="grid grid-cols-2 gap-4">
          <button
            onClick={() => cameraInputRef.current?.click()}
            className="flex flex-col items-center justify-center p-8 bg-coffee-100 rounded-xl border-2 border-dashed border-coffee-300 hover:bg-coffee-200 transition"
          >
            <Camera className="w-12 h-12 text-coffee-600 mb-2" />
            <span className="text-coffee-700 font-medium">{t('defect.camera')}</span>
          </button>
          
          <button
            onClick={() => fileInputRef.current?.click()}
            className="flex flex-col items-center justify-center p-8 bg-coffee-100 rounded-xl border-2 border-dashed border-coffee-300 hover:bg-coffee-200 transition"
          >
            <Upload className="w-12 h-12 text-coffee-600 mb-2" />
            <span className="text-coffee-700 font-medium">{t('defect.upload')}</span>
          </button>

          <input ref={cameraInputRef} type="file" accept="image/*" capture="environment" onChange={handleFileChange} className="hidden" />
          <input ref={fileInputRef} type="file" accept="image/*" onChange={handleFileChange} className="hidden" />
        </div>
      ) : (
        <div className="space-y-4">
          <img src={image} alt="Sample" className="w-full rounded-xl shadow-lg" />
          
          {loading && (
            <div className="flex flex-col items-center justify-center p-6 bg-coffee-50 rounded-xl">
              <Loader2 className="w-10 h-10 text-coffee-600 animate-spin mb-3" />
              <span className="text-coffee-700 font-medium">{loadingMsg}</span>
              <span className="text-coffee-500 text-sm mt-1">{t('defect.pleaseWait')}</span>
            </div>
          )}

          {error && (
            <div className="p-4 bg-red-50 rounded-xl">
              <div className="flex items-center text-red-700 mb-2">
                <AlertCircle className="w-5 h-5 mr-2" />
                <span className="font-medium">{error}</span>
              </div>
              <button onClick={reset} className="flex items-center text-sm text-red-600 hover:text-red-800">
                <RefreshCw className="w-4 h-4 mr-1" /> {t('defect.tryAgain')}
              </button>
            </div>
          )}

          {result && (
            <div className="card space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="font-semibold text-coffee-700">{t('defect.result')}</h2>
                <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                  (isYoloResult ? totalDefects > 0 : result.detection.is_defective)
                    ? 'bg-red-100 text-red-700' 
                    : 'bg-green-100 text-green-700'
                }`}>
                  {isYoloResult 
                    ? (totalDefects > 0 ? t('defect.defectsFound', { count: totalDefects }) : t('defect.noDefect'))
                    : (result.detection.is_defective ? t('defect.hasDefect') : t('defect.noDefect'))
                  }
                </span>
              </div>

              {isYoloResult && Object.keys(defectCounts).length > 0 && (
                <div className="space-y-2">
                  <p className="text-sm text-coffee-500 font-medium">{t('defect.defectBreakdown')}</p>
                  <div className="flex flex-wrap gap-2">
                    {Object.entries(defectCounts).map(([type, count]) => (
                      <span key={type} className={`px-3 py-1 rounded-full text-sm font-medium ${DEFECT_COLORS[type] || 'bg-gray-200'}`}>
                        {t(`defect.types.${type}`, type)}: {count}
                      </span>
                    ))}
                  </div>
                </div>
              )}

              <div className="grid grid-cols-2 gap-4">
                {isYoloResult ? (
                  <div className="p-4 bg-coffee-50 rounded-lg">
                    <p className="text-sm text-coffee-500">{t('defect.totalDefects')}</p>
                    <p className="text-2xl font-bold text-coffee-800">{totalDefects}</p>
                  </div>
                ) : (
                  <div className="p-4 bg-coffee-50 rounded-lg">
                    <p className="text-sm text-coffee-500">{t('defect.confidence')}</p>
                    <p className="text-2xl font-bold text-coffee-800">
                      {((result.detection.confidence_score ?? 0) * 100).toFixed(1)}%
                    </p>
                  </div>
                )}
                <div className="p-4 bg-coffee-50 rounded-lg">
                  <p className="text-sm text-coffee-500">{t('defect.grade')}</p>
                  <p className="text-2xl font-bold text-coffee-800">{t(`defect.grades.${result.suggested_grade}`, result.suggested_grade)}</p>
                </div>
              </div>

              <p className="text-xs text-coffee-400">
                {t('defect.processTime')}: {result.detection.processing_time_ms}ms • {result.detection.model}
              </p>
            </div>
          )}

          <button onClick={reset} className="w-full btn-primary">
            {t('defect.scanAnother')}
          </button>
        </div>
      )}
    </div>
  );
}
