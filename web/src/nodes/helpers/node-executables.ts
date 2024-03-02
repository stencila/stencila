import { ExecutionStatus, ExecutionRequired } from '@stencila/types'

type ExcutableIcon = {
  library: 'stencila' | 'default'
  name: string
}

const executeIcons: Partial<
  Record<ExecutionStatus | ExecutionRequired, ExcutableIcon>
> = {
  NeverExecuted: { library: 'default', name: 'play-circle' },
  Running: { library: 'default', name: 'pause-circle' },
  Succeeded: { library: 'default', name: 'check-circle' },
}

const executableIcon = (
  status: ExecutionStatus | undefined,
  required: ExecutionRequired
): { text: string; icon: ExcutableIcon } => {
  const fallback = { text: 'Execute', icon: executeIcons[required] }

  switch (required) {
    case 'No':
      return status === 'Succeeded'
        ? { text: 'Success, Execute Again', icon: executeIcons[status] }
        : fallback
    case 'NeverExecuted':
    default:
      return fallback
  }
}

export { executableIcon }
