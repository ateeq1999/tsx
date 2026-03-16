type Props = {
  errors?: Array<{ message?: string }>
}

export function FormErrorList({ errors }: Props) {
  if (!errors?.length) return null

  return (
    <ul className="text-sm text-destructive space-y-1">
      {errors.map((err, i) => (
        <li key={i}>{err.message}</li>
      ))}
    </ul>
  )
}