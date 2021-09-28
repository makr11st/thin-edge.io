import sys
import os
import requests
sys.path.append("apt_plugin")
from environment_apt_plugin import AptPlugin
"""
Validate apt plugin install from local file - FAIL case

Using `rolldice` package from `_ROLLDICE_URL` bellow
"""


class AptPluginInstallFromLocalFileFail(AptPlugin):
    """
    Testing that `apt` in `/etc/tedge/sm-plugins` install returns exit code 5 (Internal Error) 
    when a wrong file_path is provided
    """
    _ROLLDICE_URL = "http://ftp.br.debian.org/debian/pool/main/r/rolldice/rolldice_1.16-1+b3_amd64.deb"
    _path_to_rolldice_binary = None
    _fake_path_to_rolldice_binary = None

    def _download_rolldice_binary(self):
        # https://stackoverflow.com/questions/53101597/how-to-download-binary-file-using-requests
        local_filename = AptPluginInstallFromLocalFileFail._ROLLDICE_URL.split('/')[-1]
        current_working_directory = os.path.abspath(os.getcwd())
        self._path_to_rolldice_binary = os.path.join(current_working_directory, local_filename)
        self._fake_path_to_rolldice_binary = os.path.join(current_working_directory, "notafile.deb")

        r = requests.get(AptPluginInstallFromLocalFileFail._ROLLDICE_URL, stream=True)
        with open(self._path_to_rolldice_binary, 'wb') as f:
            for chunk in r.iter_content(chunk_size=1024): 
                if chunk: # filter out keep-alive new chunks
                    f.write(chunk)

    def setup(self):
        super().setup()
        self._download_rolldice_binary()                                # downloading the binary
        self.apt_remove("rolldice")                                     # removing just in case rolldice is already on the machine
        self.assert_isinstalled("rolldice", False)                      # asserting previous step worked
        self.addCleanupFunction(self.cleanup_remove_rolldice_binary)    # adding cleanup function to remove the binary

    def execute(self):
        """
        executing command: /etc/tedge/apt install rolldice --file `self._fake_path_to_rolldice_binary`

        this should return exit_code = 5 (internal error)
        """
        self.plugin_cmd(
                command="install",
                outputfile="outp_install",
                exit_code=5, 
                argument="rolldice", 
                file_path=f"{self._fake_path_to_rolldice_binary}")

    def validate(self):
        """
        checking that module `rolldice` is NOT installed
        """
        self.assert_isinstalled("rolldice", False)

    def cleanup_remove_rolldice_binary(self):
        """
        if we have changed the value of `_path_to_rolldice_binary` from None, then the binary was successfully downloaded in 
        ``self.setup()``, so we call os to remove it
        """
        if self._path_to_rolldice_binary:
            os.remove(self._path_to_rolldice_binary)

