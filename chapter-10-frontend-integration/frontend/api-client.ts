export type TaskDto = {
  id: string;
  title: string;
  done: boolean;
};

// @ts-ignore
export async function fetchTasks(baseUrl = ''): Promise<TaskDto[]> {
  const response = await fetch(`${baseUrl}/api/tasks`);
  if (!response.ok) {
    throw new Error(`API failed with ${response.status}`);
  }
  const body = await response.json();
  return body.tasks;
}
