import { Toast } from '@stencila/components'
import { isRPCError } from '../../client'

const toastController = Toast.toastController({
  position: Toast.ToastPositions.bottomEnd,
  type: Toast.ToastTypes.neutral,
  dismissable: true,
  duration: 8_000,
})

export const errorToast = (error: unknown) => {
  let message: string = 'Something went wrong'

  if (isRPCError(error)) {
    message = error.errors[0]?.message ?? error.message
  } else if (error instanceof Error) {
    message = error.message
  } else if (typeof error === 'string') {
    message = error
  }

  toastController.present(message, {
    type: Toast.ToastTypes.danger,
  })
}

export const showUnhandledErrors = () => {
  window.onunhandledrejection = (e: PromiseRejectionEvent) => {
    errorToast(e.reason)
  }
}
