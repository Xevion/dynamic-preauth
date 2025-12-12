import {
  Dialog,
  DialogBackdrop,
  DialogPanel,
  DialogTitle,
} from "@headlessui/react";
import { ExclamationTriangleIcon } from "@heroicons/react/24/outline";

type MobileWarningModalProps = {
  open: boolean;
  onClose: () => void;
  onContinue: () => void;
};

export default function MobileWarningModal({
  open,
  onClose,
  onContinue,
}: MobileWarningModalProps) {
  return (
    <Dialog open={open} onClose={onClose} className="relative z-50">
      <DialogBackdrop
        transition
        className="fixed inset-0 bg-black/60 backdrop-blur-sm transition-opacity duration-200 data-[closed]:opacity-0"
      />

      <div className="fixed inset-0 flex items-center justify-center p-4">
        <DialogPanel
          transition
          className="w-full max-w-sm rounded-xl border border-zinc-700 bg-zinc-900 p-5 shadow-xl transition-all duration-200 data-[closed]:scale-95 data-[closed]:opacity-0"
        >
          <div className="flex items-center gap-3 mb-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-full bg-amber-500/10">
              <ExclamationTriangleIcon className="h-5 w-5 text-amber-400" />
            </div>
            <DialogTitle className="text-lg font-semibold text-zinc-100">
              Heads up!
            </DialogTitle>
          </div>

          <p className="text-sm text-zinc-300 leading-relaxed mb-4">
            These downloads are desktop applications for Windows, macOS, and
            Linux. They won't run on mobile devices, but you're welcome to
            download them to transfer to a computer later.
          </p>

          <button
            onClick={onContinue}
            className="w-full rounded-lg bg-emerald-700 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-emerald-600 focus:outline-none focus:ring-2 focus:ring-emerald-500 focus:ring-offset-2 focus:ring-offset-zinc-900"
          >
            Got it, continue
          </button>
        </DialogPanel>
      </div>
    </Dialog>
  );
}
