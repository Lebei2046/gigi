interface ErrorStateProps {
  error: Error
  title?: string
  subtitle?: string
}

export default function ErrorState({
  error,
  title = 'Oops! Something went wrong',
  subtitle = 'Chat component error',
}: ErrorStateProps) {
  return (
    <div className="flex items-center justify-center h-full bg-gray-50 p-4">
      <div className="bg-white rounded-2xl shadow-lg border border-gray-100 p-8 max-w-md w-full text-center">
        <div className="w-16 h-16 bg-red-100 rounded-full flex items-center justify-center mx-auto mb-4">
          <svg
            className="w-8 h-8 text-red-600"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth="2"
              d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
            ></path>
          </svg>
        </div>
        <h3 className="text-xl font-semibold text-gray-900 mb-2">{title}</h3>
        <p className="text-red-600 font-medium mb-2">{subtitle}</p>
        <p className="text-sm text-gray-600 mb-6 font-mono bg-gray-50 p-3 rounded-lg text-left">
          {String(error)}
        </p>
        <button
          onClick={() => window.location.reload()}
          className="w-full py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-lg transition-colors duration-200"
        >
          Reload Application
        </button>
      </div>
    </div>
  )
}
