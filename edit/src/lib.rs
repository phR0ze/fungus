// Rather than calling out to linux functions simply handle it via Rust and yaml directives
// similar to Ansible.

// Types
// -------------------------------------------------------------------------------------------------
// ini = files
// - name: enable verbose mode
//   ini_file: dest=/etc/setting.conf section=DEFAULT option=verbose value=True backup=yes
//   tags: configuration
//
// text = unknown format, raw text
//
// regex
// - lineinfile: dest=/etc/sudoers backup=yes regexp="^{{ ATMOUSERNAME }}\\s+ALL=\\(ALL\\)\\s*ALL" line="{{ ATMOUSERNAME }} ALL=(ALL) ALL" state=present
//
// blockinfile will insert/remove multiple lines to the file.
// - name: Insert multiple lines and Backup
//   blockinfile:
//     path: /etc/ssh/sshd_config
//     backup: yes
//     block: |
//       ClientAliveInterval 360
//       ClientAliveCountMax 0
// - name: Insert after regex, backup, and validate
//   blockinfile:
//     path: /etc/ssh/sshd_config
//     backup: yes
//     marker: "# {mark} ANSIBLE MANAGED BLOCK "
//     insertbefore: '^UsePAM '
//     block: |
//       AllowUsers hakase vagrant
//       PermitEmptyPasswords no
//       PermitRootLogin no
//     validate: '/usr/sbin/sshd -T -f %s'
// - name: Remove text block surrounding by markers
//   blockinfile:
//     path: /etc/ssh/sshd_config
//     marker: "# {mark} ANSIBLE MANAGED BLOCK"
//     content: ""
//     backup: yes
//
// lineinfile is for the single line
// - name: Insert New Line under the Regex configuration
//   lineinfile:
//     path: /etc/ssh/sshd_config
//     backup: yes
//     regexp: '^PasswordAuthentication '
//     insertafter: '^#PermitEmptyPasswords '
//     line: 'PasswordAuthentication no'
//     validate: '/usr/sbin/sshd -T -f %s'
// - name: Remove a line from the file
//   lineinfile:
//     path: /etc/ssh/sshd_config
//     state: absent
//     regexp: '^PasswordAuthentication'
// - name: Replace the default
//   replace:
//     path: /etc/hosts
//     regexp: '(\s+)node\.provision\.labs(\s+.*)?$'
//     replace: '\1box.hakase.labs\2'
//     backup: yes
//
// replace module can be used to replace string
// - name: Uncomment configuration
//   replace:
//     path: /etc/nginx/nginx.conf
//     regexp: '#(\s+)server_tokens'
//     replace: 'server_tokens'
//     backup: yes
// - name: Comment Line configuration
//   replace:
//     path: /etc/nginx/nginx.conf
//     regexp: '(\s+)gzip on'
//     replace: '\n\t#gzip on'
//     backup: yes
// - name: Create Symlink of file
//   file:
//     src: /etc/nginx/sites-available/vhost
//     dest: /etc/nginx/sites-enabled/vhost
//     owner: root
//     group: root
//     state: link
// - name: Create a New Directory using file
//   file:
//     path: /etc/nginx/ssl
//     state: directory
//     owner: root
//     group: root
//     mode: 0755

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
