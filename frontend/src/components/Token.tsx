import { type ClassValue, type Platform } from "@/util";

type SessionDownloadData = {
  os: Platform;
  time: string;
  index: number;
  filename: string;
  token: string;
};

type SessionDownloadProps = {
  className?: ClassValue;
  data: SessionDownloadData;
};

// SessionDownload describes a download that occurred for the current user's session.
// It should the OS it was downloaded for, the time is was downloaded, the index (or nth), and the filename.
// It should have a button to 'remove' it from the list, too.
//
const SessionDownload = ({ data }: SessionDownloadProps) => {
  //
  const ghost = data == null;
  return <></>;
};

export default SessionDownload;
