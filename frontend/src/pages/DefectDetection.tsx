import { useState, useRef } from 'react';
import { useTranslation } from 'react-i18next';
import { Camera, Upload, Loader2, AlertCircle, CheckCircle } from 'lucide-react';
import { defectApi, DefectDetectionResult } from '../services/api';

export default function DefectDetection() {
  const { t } = useTranslation();
  const [image, setImage] = useState<string | null>(null);
  const [loading, setLoading] = useState(false);
  const [result, setResult] = useState<DefectDetectionResult | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const cameraInputRef = useRef<HTMLInputElement>(null);

  const handleFile = async (file: File) => {
    setError(null);
    setResult(null);
    
    const reader = new FileReader();
    reader.onload = async (e) => {
      const base64 = (e.target?.result as string).split(',')[1];
      setImage(e.target?.result as string);
      
      setLoading(true);
      try {
        const res = await defectApi.detect(base64);
        setResult(res);
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Detection failed');
      } finally {
        setLoading(false);
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

  return (
    <div className="space-y-6">
      <h1 className="text-2xl font-bold text-coffee-800">{t('defect.title')}</h1>

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
            <div className="flex items-center justify-center p-6 bg-coffee-50 rounded-xl">
              <Loader2 className="w-8 h-8 text-coffee-600 animate-spin mr-3" />
              <span className="text-coffee-700">{t('defect.analyzing')}</span>
            </div>
          )}

          {error && (
            <div className="flex items-center p-4 bg-red-50 rounded-xl text-red-700">
              <AlertCircle className="w-6 h-6 mr-2" />
              {error}
            </div>
          )}

          {result && (
            <div className="card space-y-4">
              <div className="flex items-center justify-between">
                <h2 className="font-semibold text-coffee-700">{t('defect.result')}</h2>
                <span className={`px-3 py-1 rounded-full text-sm font-medium ${
                  result.detection.is_defective 
                    ? 'bg-red-100 text-red-700' 
                    : 'bg-green-100 text-green-700'
                }`}>
                  {result.detection.is_defective ? t('defect.hasDefect') : t('defect.noDefect')}
                </span>
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div className="p-4 bg-coffee-50 rounded-lg">
                  <p className="text-sm text-coffee-500">{t('defect.confidence')}</p>
                  <p className="text-2xl font-bold text-coffee-800">
                    {(result.detection.confidence_score * 100).toFixed(1)}%
                  </p>
                </div>
                <div className="p-4 bg-coffee-50 rounded-lg">
                  <p className="text-sm text-coffee-500">{t('defect.grade')}</p>
                  <p className="text-2xl font-bold text-coffee-800">{result.suggested_grade}</p>
                </div>
              </div>

              <div className="p-4 bg-amber-50 rounded-lg">
                <p className="text-sm text-amber-700">
                  <strong>Note:</strong> {result.detection.note}
                </p>
              </div>

              <p className="text-xs text-coffee-400">
                Model: {result.detection.model} | Time: {result.detection.processing_time_ms}ms
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
