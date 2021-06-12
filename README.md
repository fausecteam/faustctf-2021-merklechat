Service Template
================

This repository should give you a general idea for how to set up your service repository, do Continuous
Integration (CI) and install the service on the Vulnbox.

General Remarks
---------------
**See [Christoph's slides](https://faust.cs.fau.de/files/faustctf-2018_service-howto.pdf), the most important
points:**

* Services should be solvable by *good* teams within eight hours
* We don't want cross-service exploits to be possibe
* Everything runs on Debian stretch
* Service structure
	* Everything is started via systemd, presumably with resource limits and some sandboxing
	* Socket services may be written inetd-style (communication through stdin and stdout)
	* uwsgi, fcgi, php-fpm etc. can be made available through nginx
* Checker scripts
	* Check whether the service is available and flags from previous round can be obtained
	* Good: Adding randomization within your protocol (layer 5+), fingerprinting at lower layers will be
	  prohibited by us
	* Consider scalability (doing expensive stuff in the checker can be problematic)
	* Storing data in the file system is not possible, but we provide a key/value store
	* **Don't underestimate the complexity of writing a checker**
	* **Add lots of logging output**
* Example exploit
	* One per vulnerability
	* Should extract flags reliably from an unpatched Vulnbox

Core Services
-------------
### Build
* [GitLab CI](https://docs.gitlab.com/ce/ci/README.html) is used to be the service on every commit
* "Build" in this context means:
	* Every step from source to running service that does not have to run on the Vulnbox
	* Compilation, maybe also stuff like creating a virtualenv
	* Possibly also unit tests, if you have them :-)
* **If you need (large binary) files for your build which cannot be obtained otherwise, do not add them to
  the Git repository.** Instead, talk to us and we'll host them on our web server.
* There should be a top-level Makefile
	* … whose `install` target puts required files into a Unix file system structure within "dist_root" in
	  the projects base directory
	* Inside "dist_root", service files reside in "/srv/&lt;service_slug&gt;"
	* Obviously, the contents of "dist_root" will be copied the Vulnbox's root directory during installation
* Add a [".gitlab-ci.yml" file](https://docs.gitlab.com/ce/ci/yaml/)
	1. Build job
		* It should run in a Docker container, specify the Docker image to use through `image`
			* Search for images on [Docker Hub](https://hub.docker.com)
			* **Please, only use official images** (without "&lt;username&gt;/" in their name)
			* If possible, use the right variant (tag) of an image to prevent library version mismatches, i.e.
			  something like "stretch"
			* In doubt, use a plain Debian images ("debian:stretch") and install stuff through Apt
		* Prepare the system (e.g. install dependencies) using `before_script`
		* Invoke `make install` using `script`
		* Store "metadata.yml" and "dist_root/" as `artifacts` `paths`
		* Probably you want to build `only` on the "master" branch
		* Set `tags` "faust" and "docker" to ensure the right Runner is used
	2. Upload job
		* Uploads the `artifacts` from the build job to our web server, where the Vulnbox build will pick
		  them up[^1]
		* The special "FAUST WWW Uploader" Runner (not using containers) has an SSH key to upload files to
		  *www.faust.cs.fau.de*
		* `script`
			1. Create the target directory "/var/www/files/internal/ci/faustctf/2018/&lt;service_slug&gt;"
			   using `ssh ci-upload@www.faust.cs.fau.de`
			2. Create a TAR archive of "dist_root" called "dist_root.tar.gz", making sure that the contents
			   of "dist_root" are at the root level (no "dist_root" directory inside the archive)
			3. Upload "metadata.yml" and "dist_root.tar.gz" to the target directory using `scp
			  ci-upload@www.faust.cs.fau.de`
		* You have to manually clean up the working directory (remove all files from `CI_PROJECT_DIR`) with
		  an `after_script`
		* It should probably `only` run on the "master" branch as well
		* Set `tags` "faust" and "www-upload" to ensure the special Runner is used

[^1]: This additional step is required because [multi-project Pipelines](https://docs.gitlab.com/ee/ci/multi_project_pipeline_graphs.html) are a GitLab Premium feature and the Vulnbox build therefore cannot access service build artifacts easily.

### Installation
Installation on the actual Vulnbox happens in four steps:

1. The contents stored in "dist_root" during the build are extracted
	* To the root filesystem
	* Files created this way will be owned by root, run `chown` using `postinst_commands` if you need
	  different owners
2. Debian packages are installed
	* As specified under `install`/`debian_packages` in "metadata.yml"
	* Maybe consider adding some build dependencies as well, so that teams can more easily patch the service
3. The user account to run the service under is created with the service's slug as username
4. Arbitrary commands from `install`/`postinst_commands` are run as root
	* They run in a shell, so you can make use of shell features
	* The environment variable `DATA_DIR` contains a directory where the service is supposed to store its
	  (sensitive) working data

Checkers
--------
For the checker script, you can either have a directory that also is a Python module (as in the example here)
or a single Python file:

    checker/
     template.py

Or:

    checker/
      template
      ├── __init__.py
      ├── mychecker.py
      ├── someimage.jpg
      ├── somescript.pl
      └── somfile.json

If you need any auxiliary data, use the Python module method and place the data in that directory. The one
and only language is Python 3. You can still use other languages after consulting with the Organizing Team by
placing a script in there and calling it from Python.

In any way, the required interface is:

    from $servicename import ServiceNameChecker

Ohh, and did we mention that you should **add logging**?
