export default function MnemonicConfirm() {
  return (
    <div className="p-8 bg-white">
      <div>
        <h1 className="text-2xl font-bold mb-4">Confirm phrase</h1>
        <p className="mb-4 text-gray-600">
          Write your phrase again to ensure you wrote it down correctly.
        </p>
      </div>
      <div className="mb-6">
        <div className="grid grid-cols-3 gap-2">
          {Array.from({ length: 12 }).map((_, index) => {
            const isInput = [2, 5, 9].includes(index); // Random indices: 2, 5, 9
            return (
              <div key={index} className="p-2 text-left">
                <span className="text-gray-500 text-right w-8 inline-block">{index + 1}.</span>
                {isInput ? (
                  <input
                    type="text"
                    className="ml-2 px-2 py-1 w-24 border-b border-gray-300"
                    placeholder="Enter word"
                  />
                ) : (
                  <span className="ml-2 font-medium border-b border-gray-300 ">word</span>
                )}
              </div>
            );
          })}
        </div>

      </div>


      <div className="flex justify-between">
        <button className="btn btn-outline w-40">Back</button>
        <button className="btn btn-primary w-40">Continue</button>
      </div>
    </div>
  );
}
