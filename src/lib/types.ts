export type NativeWindow = {
  hwnd: number;
  title: string;
  process_id: number;
  process_name?: string;
  icon_base64?: string;
};
