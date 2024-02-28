import { ExecutionStatus, ExecutionRequired } from '@stencila/types'

type ExcutableIcon = {
  iconLibrary: 'stencila' | 'default'
  icon: string
}

const execStatusIcons: Partial<
  Record<ExecutionStatus | ExecutionRequired, ExcutableIcon>
> = {
  NeverExecuted: { iconLibrary: 'default', icon: 'play-circle' },
  Running: { iconLibrary: 'default', icon: 'pause-circle' },
}

const executableIcon = (status: ExecutionStatus | ExecutionRequired) => {
  return execStatusIcons[status]
}

export { executableIcon }
